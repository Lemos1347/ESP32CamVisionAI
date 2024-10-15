package main

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"text/template"
)

// Config represents the structure of the TOML file.
type Config struct {
	WifiName        string
	WifiPassword    string
	ServerURL       string
	Flash           bool
	FlashBrightness int
}

// Prompt the user for input and return the entered string.
func promptInput(prompt string) string {
	reader := bufio.NewReader(os.Stdin)
	fmt.Print(prompt)
	input, _ := reader.ReadString('\n')
	return strings.TrimSpace(input)
}

// Create a TOML file using the user's input.
func createTomlFile(config Config, outputPath string) {
	// Define the TOML structure.
	const tomlTemplate = `[esp32cam_rs]
wifi_ssid = "{{.WifiName}}"
wifi_psk = "{{.WifiPassword}}"
server_url = "{{.ServerURL}}"
use_flash = {{.Flash}}
flash_brightness = {{.FlashBrightness}}
  `
	// Create a new file at the specified path.
	file, err := os.Create(outputPath)
	if err != nil {
		log.Fatalf("Failed to create TOML file: %v", err)
	}
	defer file.Close()

	// Parse the template and execute it.
	tmpl, err := template.New("config").Parse(tomlTemplate)
	if err != nil {
		log.Fatalf("Failed to parse TOML template: %v", err)
	}

	err = tmpl.Execute(file, config)
	if err != nil {
		log.Fatalf("Failed to write to TOML file: %v", err)
	}

	fmt.Printf("TOML file '%s' created successfully.\n", outputPath)
}

// Run a shell command from the specified directory (e.g., cargo build or espflash).
func runCommandFromDir(dir string, command string, args ...string) {
	// Set the working directory.
	cmd := exec.Command(command, args...)
	cmd.Dir = dir
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	err := cmd.Run()
	if err != nil {
		log.Fatalf("Command failed: %v", err)
	}
}

func main() {
	// Step 1: Prompt user for WiFi name, WiFi password, and Server URL.
	wifiName := promptInput("Enter WiFi name: ")
	wifiPassword := promptInput("Enter WiFi password: ")
	serverURL := promptInput("Enter Server URL: ")
	flashAnswer := promptInput("Do you want to use flash?(y/N) ")

	var flash bool
	flashBrightness := 32
	if flashAnswer == "y" {
		flash = true
	} else {
		flash = false
		flashBrightness, _ = strconv.Atoi(promptInput("Which intensity would you like? (0 - 255) "))
	}

	// Step 2: Define paths.
	embeddedDir := os.Getenv("EMBEDDED_DIR")
	if embeddedDir == "" {
		embeddedDir = "./embedded"
	}

	tomlFilePath := os.Getenv("TOML_FILE_PATH")
	if tomlFilePath == "" {
		tomlFilePath = "./embedded/cfg.toml" // Absolute path to the TOML file.
	}
	// Create a Config struct with the input values.
	config := Config{
		WifiName:        wifiName,
		WifiPassword:    wifiPassword,
		ServerURL:       serverURL,
		Flash:           flash,
		FlashBrightness: flashBrightness,
	}

	// Step 3: Create the TOML file in the /embedded directory.
	createTomlFile(config, tomlFilePath)

	// Step 4: Compile the Rust application using Cargo from the /embedded directory.
	fmt.Println("Compiling the Rust application with 'cargo build' in the /embedded directory...")
	runCommandFromDir(embeddedDir, "cargo", "build", "--release")
	//
	// // Step 5: Flash the compiled binary using espflash.
	fmt.Println("Flashing the binary to the ESP microcontroller with 'espflash'...")
	binaryPath := filepath.Join(embeddedDir, "/target/xtensa-esp32-espidf/release/esp32cam_rs") // Adjust with your project's binary name.
	runCommandFromDir(embeddedDir, "espflash", "flash", binaryPath)
	//
	fmt.Println("Application successfully flashed to the ESP device.")
}
