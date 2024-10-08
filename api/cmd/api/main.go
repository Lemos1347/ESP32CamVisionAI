package main

/*
#cgo LDFLAGS: -L./lib -lyolo_rs -ldl -lm -lstdc++
#include "bindings.h"
*/
import (
	"C"
)

import (
	"encoding/base64"
	"fmt"
	"image"
	"image/jpeg"
	"io"
	"log"
	"net/http"
	"os"
	"runtime"
	"sync"
	"time"
	"unsafe"
)

var (
	models      []*C.YOLOv8
	modelsMutex = sync.Mutex{}
	modelsCond  = sync.NewCond(&modelsMutex)
)

const bufferSize = 2

type ImageBuffer struct {
	mu     sync.Mutex
	buffer [][]byte
	index  int
	ch     chan []byte
}

func NewImageBuffer() *ImageBuffer {
	return &ImageBuffer{
		buffer: make([][]byte, bufferSize),
		index:  0,
		ch:     make(chan []byte, bufferSize),
	}
}

func (ib *ImageBuffer) Add(imagePath *string) {
	ib.mu.Lock()
	defer ib.mu.Unlock()

	file, err := os.Open(*imagePath)
	if err != nil {
		return
	}
	defer file.Close()

	image, img_err := io.ReadAll(file)
	if img_err != nil {
		return
	}

	ib.buffer[ib.index] = image
	ib.index = (ib.index + 1) % bufferSize

	// Send image to channel
	select {
	case ib.ch <- image:
		// Success to send to chn
		log.Println("Image sent to channel")
	default:
		// Chn is full, descart the oldest photo
		log.Println("Channel is full")
	}
}

func (ib *ImageBuffer) GetChannel() <-chan []byte {
	return ib.ch
}

var imgBuffer = NewImageBuffer()

func saveImage(fileName string, fileData io.Reader) error {
	out, err := os.Create(fileName)
	if err != nil {
		return err
	}
	defer out.Close()

	img, _, err := image.Decode(fileData)
	if err != nil {
		return err
	}

	return jpeg.Encode(out, img, nil)
}

func handlePost(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Método não suportado", http.StatusMethodNotAllowed)
		return
	}

	err := r.ParseMultipartForm(10 << 20) // 10 MB
	if err != nil {
		http.Error(w, "Erro ao fazer o parse do form-data", http.StatusBadRequest)
		return
	}

	file, _, err := r.FormFile("file")
	if err != nil {
		log.Printf("Content-lenght: %d", r.ContentLength)
		log.Printf("Err: %s\n", err.Error())
		http.Error(w, "Erro ao obter o arquivo", http.StatusBadRequest)
		return
	}
	defer file.Close()

	fileBuffer, err := io.ReadAll(file)
	if err != nil {
		log.Printf("Erro ao ler o arquivo: %s\n", err.Error())
		http.Error(w, "Erro ao ler o arquivo", http.StatusInternalServerError)
		return
	}

	go func(models []*C.YOLOv8, modelsMutex *sync.Mutex, modelsCond *sync.Cond, img_buffer *ImageBuffer) {
		modelsMutex.Lock()

		for len(models) == 0 {
			modelsCond.Wait()
		}

		model := models[len(models)-1]
		if model == nil {
			log.Println("Unable to process image, model's pointer is nil")
			return
		}
		// Remove the last available model
		models = models[:len(models)-1]
		// Returns the model back on
		defer func() {
			models = append(models, model)
			modelsMutex.Unlock()
		}()

		cBuffer := (*C.uint8_t)(unsafe.Pointer(&fileBuffer[0]))

		result := C.process_image(model, cBuffer, C.int(len(fileBuffer)))
		if result == nil {
			log.Println("Error while processing image!")
		}

		file_name := C.GoString(result)
		defer C.free_c_string(result)

		log.Printf("File created: %s\n", file_name)

		img_buffer.Add(&file_name)

	}(models, &modelsMutex, modelsCond, imgBuffer)

	log.Println("Image received!")
	w.Header().Set("Content-Type", "application/json")
	w.Write([]byte(`{"message": "Imagem salva com sucesso!"}`))
}

func streamHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")

	imageChan := imgBuffer.GetChannel()

	for {
		select {
		case img := <-imageChan:
			// Convert image to base64
			encodedImage := base64.StdEncoding.EncodeToString(img)
			fmt.Fprintf(w, "data:image/jpeg;base64,%s\n\n", encodedImage)
			w.(http.Flusher).Flush()
		case <-time.After(100 * time.Millisecond):
			// Timeout to not block loop
		}
	}
}

func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*") // Permite qualquer origem
		w.Header().Set("Access-Control-Allow-Methods", "POST, GET, OPTIONS, PUT, DELETE")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type")

		if r.Method == http.MethodOptions {
			return
		}

		next.ServeHTTP(w, r)
	})
}

func main() {
	numCPUs := runtime.NumCPU() - 1
	if numCPUs < 1 {
		numCPUs = 1
	}

	model_path := C.CString("./assets/YoloV8n-Face.onnx")
	saving_dir := C.CString("./assets/results")
	defer C.free(unsafe.Pointer(model_path))
	defer C.free(unsafe.Pointer(saving_dir))

	for i := 0; i < numCPUs; i++ {
		model := C.load_model(model_path, saving_dir)
		if model == nil {
			log.Fatalln("Load model returned a nil pointer!")
		}
		models = append(models, model)
		log.Printf("Model %d loaded in Go! %#v\n", i+1, model)
	}

	defer func() {
		for _, model := range models {
			C.free_model(model)
		}
	}()

	mux := http.NewServeMux()

	// Cria uma rota POST
	mux.HandleFunc("/post", handlePost)
	mux.HandleFunc("/stream", streamHandler)

	handlerWithCors := corsMiddleware(mux)

	fmt.Println("Servidor rodando na porta 8080...")
	log.Fatal(http.ListenAndServe(":8080", handlerWithCors))
}
