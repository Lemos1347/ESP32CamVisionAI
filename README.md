# Sistema de Detecção Facial com ESP32 e Processamento Remoto

## Introdução

Este repositório contém o código para um sistema de visão computacional que utiliza um microcontrolador ESP32-CAM para capturar imagens e enviá-las para um servidor remoto, onde são processadas para detecção de faces humanas. O objetivo é implementar uma solução em tempo real que processe e exiba as imagens com a detecção das faces marcadas.

A solução é dividida em duas partes principais: o ESP32-CAM responsável pela captura das imagens e um servidor que processa as imagens recebidas e exibe o resultado em um fluxo contínuo (SSE - Server-Sent Events).

## Funcionamento

https://github.com/user-attachments/assets/0d62bd79-1d2c-4ff2-a35c-c9f389a0fab8

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
  - [yolo_rs](./api/cmd/lib/yolo_rs): Contém todo um pacote rust para fazer a interface com um modelo ONNX.
- [assets](/api/assets): Contém o modelo Yolov8 de detecção de faces humanas e os resultados do modelo.

#### 2.1 Arquitetura e Funcionamento em Nível de Aplicação

**Processo Geral**:

1. **Recepção de Imagens**: O servidor Go expõe uma rota `/post` que recebe requisições HTTP POST contendo imagens enviadas pelo ESP32-CAM.

2. **Processamento de Imagens**:

   - **Integração Go e Rust via CGO**: Ao receber uma imagem, o servidor Go utiliza o CGO para chamar funções implementadas em Rust. O CGO (C Go) permite que o código Go interaja com bibliotecas escritas em outras linguagens que seguem convenções C, neste caso, Rust.

   - **Rust e FFI (Foreign Function Interface)**: No lado do Rust, utilizamos o FFI para expor funções com interface C, permitindo que sejam chamadas a partir do Go. As funções Rust são anotadas com `#[no_mangle]` e definidas como `extern "C"` para garantir compatibilidade.

   - **Processamento com YOLOv8**: O código Rust carrega o modelo YOLOv8 (formato ONNX) para detecção de faces. Ele processa a imagem recebida, detecta faces humanas e salva a imagem resultante com as detecções marcadas em um diretório específico.

3. **Armazenamento em Buffer Circular**: As imagens processadas são armazenadas em um buffer circular implementado em Go, que mantém as duas imagens mais recentes. Isso otimiza o uso de memória e garante que as imagens mais recentes estejam sempre disponíveis para visualização.

4. **Streaming para Clientes**: O servidor Go também expõe uma rota `/stream` que permite que clientes se conectem e recebam as imagens processadas em tempo real via SSE (Server-Sent Events), possibilitando a visualização contínua das detecções em uma interface web.

#### 2.2 Detalhes da Integração entre Go e Rust

**No Lado do Rust**:

- **Exposição de Funções via FFI**: As funções em Rust que precisam ser acessíveis a partir do Go são definidas com a convenção de chamada C e anotadas com `#[no_mangle]`:

  ```rust
  #[no_mangle]
  pub extern "C" fn load_model(c_model_path: *const c_char, c_save_dir: *const c_char) -> *mut YOLOv8 { /* ... */ }
  ```

- **Interface C Compatível**: As funções utilizam tipos compatíveis com C, como `*const c_char` para strings e ponteiros para estruturas.

- **Criação de Biblioteca Estática**: O código Rust é compilado em uma biblioteca estática (`libyolo_rs.a`) que pode ser linkada ao código Go.

**No Lado do Go**:

- **Uso do CGO para Chamada de Funções C**: No código Go, utilizamos diretivas `import "C"` para indicar que estamos usando o CGO, e incluímos as diretivas de compilação para linkar a biblioteca estática do Rust:

  ```go
  /*
  #cgo LDFLAGS: -L./lib/yolo_rs/target/release -lyolo_rs -lpthread -ldl -lm -lstdc++
  #include "bindings.h"
  */
  import "C"
  ```

- **Chamada das Funções Rust**: As funções expostas pelo Rust são chamadas como se fossem funções C no Go:

  ```go
  model = C.load_model(model_path, saving_dir)
  ```

- **Gerenciamento de Ponteiros e Memória**: O código Go cuida da conversão de tipos e gerenciamento de ponteiros conforme necessário para interagir corretamente com o código Rust.

#### 2.3 Motivação e Benefícios da Abordagem

**Vantagens de Combinar Go e Rust**:

- **Go para Conectividade e Concorrência**:

  - Go é excelente para construir servidores web eficientes, com suporte nativo à concorrência através de goroutines.
  - Facilita a implementação de servidores HTTP, manipulação de requisições e streaming de dados para clientes.

- **Rust para Desempenho e Segurança**:

  - Rust oferece alto desempenho e segurança de memória, sendo ideal para processamento intensivo como a detecção de faces em imagens.
  - Gerencia recursos de forma eficiente, evitando problemas comuns de gerenciamento de memória.

**Por que Utilizar CGO e FFI**:

- **Integração de Componentes**: O CGO permite que o código Go chame funções escritas em outras linguagens que seguem a convenção C, como o Rust via FFI. Isso permite combinar o melhor de ambos os mundos.

- **Reutilização de Código**: Aproveita bibliotecas e funcionalidades existentes em Rust, sem precisar reescrever toda a lógica em Go.

- **Desempenho**: Chamar código nativo otimizado em Rust a partir de Go pode melhorar significativamente o desempenho para tarefas específicas. Além disso, como a biblioteca Rust é incorporada diretamente ao binário final durante o processo de compilação, não há gargalo de performance associado às chamadas de funções externas. Isso significa que o processamento ocorre de forma eficiente, sem overhead adicional, garantindo rapidez na detecção de faces.

#### 2.4 Linkagem Estática e Sua Importância

**O que é Linkagem Estática**:

- **Linkagem Estática**: Processo de incorporação de todas as bibliotecas e dependências diretamente no executável durante a compilação. O resultado é um binário autossuficiente.

- **Linkagem Dinâmica**: As bibliotecas são vinculadas em tempo de execução. O executável depende da presença das bibliotecas corretas no sistema onde é executado.

**Por que a Linkagem Estática é Importante neste Projeto**:

- **Portabilidade**:

  - Garante que o binário possa ser executado em diferentes ambientes sem necessidade de instalar dependências adicionais.
  - Facilita a distribuição e implantação da aplicação, especialmente em contêineres Docker ou sistemas embarcados.

- **Simplificação do Deploy**:

  - Elimina problemas relacionados a versões incompatíveis de bibliotecas ou ausência de dependências no ambiente de produção.
  - Reduz a complexidade do ambiente de execução, pois não depende de bibliotecas externas.

- **Integração Go e Rust**:

  - Assegura que a biblioteca estática do Rust esteja corretamente incluída no binário Go.
  - Evita problemas de linkagem dinâmica que podem ocorrer devido a incompatibilidades entre diferentes sistemas ou configurações.

**Diferenças entre Linkagem Estática e Dinâmica**:

- **Linkagem Estática**:

  - **Vantagens**:
    - Executável independente.
    - Maior controle sobre o ambiente de execução.
    - Elimina dependências externas em tempo de execução.
  - **Desvantagens**:
    - Tamanho maior do executável.
    - Menos flexibilidade para atualizar bibliotecas sem recompilar.

- **Linkagem Dinâmica**:

  - **Vantagens**:
    - Executáveis menores.
    - Possibilidade de atualizar bibliotecas sem recompilar o executável.
  - **Desvantagens**:
    - Dependência de bibliotecas externas em tempo de execução.
    - Possibilidade de conflitos de versão ou ausência de bibliotecas necessárias.

##### 2.4.1 Importante: Considerações sobre a Linkagem Estática no macOS

É fundamental destacar que a flag `-static` não é incluída diretamente nas diretivas do CGO (`#cgo LDFLAGS`) no arquivo Go. Em vez disso, a linkagem estática é aplicada durante o processo de compilação no Dockerfile, por meio do comando de build. Isso ocorre devido a uma limitação conhecida no macOS, que não disponibiliza uma versão estática da biblioteca `libSystem.dylib`, essencial para a linkagem estática completa.

Conforme explicado por um usuário no stackoverflow:

> _"Você pode compilar `crt0.o` via Csu (abreviação de 'C start up'), mas infelizmente esse `crt0.o` é incapaz de linkar com `libc`, já que não há uma versão estática de `libSystem.dylib`. Portanto, não é suportado até que a Apple nos forneça uma versão estática de `libSystem.dylib`. Ou isso, ou não usar `libc`. Há mais detalhes neste ticket do Github para Csu."_

Além disso, a documentação do `gcc` reforça essa limitação:

> \*"-static  
> Em sistemas que suportam linkagem dinâmica, isso previne a linkagem com as bibliotecas compartilhadas. Em outros sistemas, esta opção não tem efeito.
>
> Esta opção não funcionará no macOS a menos que todas as bibliotecas (incluindo `libgcc.a`) também tenham sido compiladas com `-static`. Como nem uma versão estática de `libSystem.dylib` nem `crt0.o` são fornecidos, esta opção não é útil para a maioria das pessoas."\*

**Por que isso é relevante para o projeto?**

- **Limitações do macOS**: Devido à ausência de uma versão estática da `libSystem.dylib` no macOS, tentar realizar a linkagem estática diretamente nas flags do CGO resultaria em erros de compilação ou em um binário incompatível.
- **Solução via Docker**: Ao executar o processo de build dentro de um contêiner Docker com um sistema operacional que suporta a linkagem estática completa (como Alpine Linux ou outras distribuições Linux), é possível incluir a flag `-static` durante a compilação, garantindo que o binário resultante seja autossuficiente e portátil.
- **Portabilidade**: Compilar o binário com linkagem estática em um ambiente controlado assegura que ele possa ser executado em diferentes sistemas, sem depender das bibliotecas dinâmicas do sistema host.

**Referências:**

- [How to static link on OS X](https://stackoverflow.com/questions/844819/how-to-static-link-on-os-x)
- [Issue regarding static linking in Csu-85](https://github.com/skaht/Csu-85/issues/2)
- [Creating static Mac OS X C++ build](https://stackoverflow.com/questions/5259249/creating-static-mac-os-x-c-build)

#### 2.5 Resumo do Processo em Nível de Aplicação

1. **Construção da Biblioteca Rust**:

   - O código Rust é compilado em uma biblioteca estática (`libyolo_rs.a`).
   - As funções necessárias são expostas via FFI com interface C.

2. **Configuração do CGO no Go**:

   - O Go é instruído a linkar a biblioteca estática do Rust utilizando as diretivas de compilação no cabeçalho do arquivo Go.
   - O cabeçalho C (`bindings.h`) é incluído para que o Go conheça as assinaturas das funções.

3. **Compilação com Linkagem Estática**:

   - Durante a compilação, todas as dependências (incluindo a biblioteca Rust) são incorporadas no binário final.
   - A flag `-static` é utilizada para garantir a linkagem estática.

4. **Execução da Aplicação**:

   - O binário resultante pode ser executado em qualquer ambiente compatível sem necessidade de bibliotecas adicionais.
   - O servidor Go inicia, carrega o modelo de detecção de faces e começa a processar as imagens recebidas.

### 3. Frontend: Visualização das Imagens

A visualização das imagens é feita através de uma página [HTML](./frontend) que se conecta ao servidor e recebe o stream de imagens via SSE.

## Como Executar

> [!IMPORTANT]
> Para esse projeto é necessario ter configurado em sua máquina: [Rust](https://www.rust-lang.org/) e [Golang](https://go.dev/).  
> Para conseguir utilizar o pacote do embarcado, é necessário seguir as instruções do [rust hal para esp32](https://docs.esp-rs.org/book/introduction.html) e também baixar a biblioteca [esp32-camera](https://github.com/espressif/esp32-camera).  
> Para conseguir utilizar a API é crucial que o seu clang esteja devidamente configurado em seu dispositivo.  
> Para alterar as credenciais do wifi e qualquer outra configuração do esp32, crie um arquivo `cfg.toml` com os mesmo campos que em [cfg.toml.example](./embedded/cfg.toml.example)

Após toda a configuração inicial, para rodar tanto o embarcado quando a api há Justfiles.

_API_

```bash
just app
```

_Embarcado_

```bash
just esp32cam
```
