# Sistema de Detecção Facial com ESP32 e Processamento Remoto

## Introdução

Este repositório contém o código para um sistema de visão computacional que utiliza um microcontrolador ESP32-CAM para capturar imagens e enviá-las para um servidor remoto, onde são processadas para detecção de faces humanas. O objetivo é implementar uma solução em tempo real que processe e exiba as imagens com a detecção das faces marcadas.

A solução é dividida em duas partes principais: o ESP32-CAM responsável pela captura das imagens e um servidor que processa as imagens recebidas e exibe o resultado em um fluxo contínuo (SSE - Server-Sent Events).

## Funcionamento

https://github.com/user-attachments/assets/fcf8186e-c147-4547-802e-977a5c97fd40

1. O ESP32-CAM captura imagens em tempo real.
2. As imagens são enviadas para um servidor remoto para processamento.
3. O servidor processa as imagens utilizando um algoritmo de detecção de faces.
4. As duas imagens mais recentes são armazenadas em um buffer circular.
5. Um fluxo de imagens é enviado para o cliente através de SSE (Server-Sent Events), permitindo a visualização das imagens com as faces detectadas.

## Arquitetura da Solução

A solução é composta por duas partes principais: o microcontrolador ESP32-CAM e o servidor de processamento de imagens.

### 1. ESP32-CAM: Captura de Imagens

O ESP32-CAM é utilizado para capturar imagens em tempo real. Ele é programado para enviar essas imagens via rede para o servidor de processamento.  
Dentro da pasta [emedded](./embedded) está o código utilizado no embarcado, o qual está organizado da seguinte forma:

- [esp32-camera](/embedded/esp32-camera): Contém o código da lib para a câmera do ESP32.
- [src](/embedded/src): Contém o código principal do embarcado.
  - [lib.rs](/embedded/src/lib.rs): Declaração e implementação de todos os modules.
  - [main.rs](/embedded/src/main.rs): Função principal do embarcado.
  - [configs](/embedded/src/configs): Todos os arquivos de configuração de alguns periféricos do embarcado.
  - [utils](/embedded/src/utils): Funções utilitárias para o embarcado, como a criação de um form-data.
- [cfg.toml.example](/embedded/cfg.toml.example): Template de arquivo de configuração de variáveis, como wifi ssid e password.

### 2. Servidor de Processamento: Detecção de Faces

O servidor é responsável por receber as imagens do ESP32-CAM, processá-las para identificar faces humanas e armazenar as duas imagens mais recentes em um buffer circular.  
Dentro da pasta [api](./api) está o seu código, o qual está organizado da seguinte forma:

- [cmd](./api/cmd): Contém todas as aplicações, nesse caso um api em golang e um lib em rust.
   - [api](./api/cmd/api): Contém o arquivo principal da aplicação que contempla o header file do cgo, as rotas e as suas respectivas lógicas.
   - [yolo_rs](./api/cmd/yolo_rs): Contém todo um pacote rust para fazer a interface com um modelo ONNX.
- [assets](/api/assets): Contém o modelo Yolov8 de detecção de faces humanas e os resultados do modelo.


### 3. Frontend: Visualização das Imagens

A visualização das imagens é feita através de uma página [HTML](./frontend) que se conecta ao servidor e recebe o stream de imagens via SSE.

## Como Executar

> [!IMPORTANT]
> Para esse projeto é necessario ter configurado em sua máquina: [Rust](https://www.rust-lang.org/) e [Golang](https://go.dev/).
> Para conseguir utilizar o pacote do embarcado, é necessário seguir as instruções do [rust hal para esp32](https://docs.esp-rs.org/book/introduction.html) e também baixar a biblioteca [esp32-camera](https://github.com/espressif/esp32-camera).
> Para conseguir utilizar a API é crucial que o seu clang esteja devidamente configurado em seu dispositivo.

Após toda a configuração inicial, para rodar tanto o embarcado quando a api há Justfiles.

*API*
```bash
just app
```
*Embarcado*
```bash
just esp32cam
```
