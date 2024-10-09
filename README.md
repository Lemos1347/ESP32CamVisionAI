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

#### 1.1 Uso do FreeRTOS no ESP32-CAM

Está sendo utilizado um sistema operacional em tempo real, FreeRTOS, no ESP32-CAM para gerenciar tarefas e recursos, permitindo a execução concorrente de múltiplas tarefas em um ambiente de microcontrolador. No nosso projeto, aproveitamos as funcionalidades do FreeRTOS de forma indireta, utilizando as abstrações fornecidas pelo Rust e pelas crates específicas para o ESP32.

##### 1.1.1 Execução Paralela: Captura e Envio de Imagens

Implementamos duas threads principais:

1. **Thread de Captura de Imagens**: Responsável por capturar imagens continuamente da câmera.
2. **Thread de Envio de Imagens**: Responsável por enviar as imagens capturadas ao servidor remoto.

Essa abordagem permite que a captura de imagens não seja bloqueada pelo tempo de envio, garantindo maior eficiência e responsividade do sistema. Ao separar essas tarefas, o microcontrolador pode continuar capturando imagens enquanto outra thread se ocupa do envio, otimizando o uso dos recursos disponíveis.

##### 1.1.2 Sincronização entre Threads

Para coordenar a interação entre as threads, utilizamos as seguintes primitivas de sincronização do Rust:

- **`Arc` (Atomic Reference Counting)**: Permite compartilhamento seguro de dados entre threads.
- **`Mutex`**: Garante acesso exclusivo ao buffer compartilhado de imagens, prevenindo condições de corrida.
- **`Condvar` (Condition Variable)**: Permite que uma thread aguarde até que uma condição específica seja atendida, facilitando a comunicação entre threads.

O fluxo de sincronização funciona da seguinte forma:

- **Captura de Imagens**: A thread de captura adiciona imagens ao buffer compartilhado e notifica a thread de envio através da `Condvar`.
- **Envio de Imagens**: A thread de envio aguarda notificações. Ao ser notificada, verifica se há imagens no buffer, remove a imagem mais antiga e a envia ao servidor.
- **Coordenação**: O uso de `Mutex` garante que apenas uma thread acesse o buffer por vez, enquanto `Condvar` sincroniza a produção e consumo das imagens.

Essa sincronização assegura que as imagens sejam processadas de forma ordenada e eficiente, evitando perdas ou sobreposições.

##### 1.1.3 Utilização de `std::thread` e Considerações sobre FreeRTOS

Embora o FreeRTOS gerencie as tarefas em nível de sistema, utilizamos a API padrão do Rust, `std::thread`, para criação de threads. Essa escolha traz os seguintes benefícios:

- **Abstração e Simplicidade**: A API do Rust oferece uma interface mais amigável e segura, permitindo que os desenvolvedores foquem na lógica da aplicação sem se preocupar com detalhes do sistema operacional subjacente.
- **Segurança**: O Rust previne erros comuns em programação concorrente, como condições de corrida e deadlocks, através de seu sistema de tipos e verificações em tempo de compilação.
- **Compatibilidade**: `std::thread` é suportada no ambiente do ESP32 e mapeia internamente para tarefas do FreeRTOS, garantindo desempenho e eficiência.

A documentação do `esp-idf-hal` reforça essa abordagem:

> **esp-idf-hal/task.rs**: pub unsafe fn create
>
> This API is to be used only for niche use cases like where the `std` feature is not enabled, or one absolutely needs to create a raw FreeRTOS task.
>
> In all other cases, the standard, safe Rust `std::thread` API should be utilized, as it is anyway a thin wrapper around the FreeRTOS task API.

[Referência: `esp-idf-hal` task.rs](https://github.com/esp-rs/esp-idf-hal/blob/master/src/task.rs)

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

#### 2.4 Linkagem Dinâmica e Sua Importância

**O que é Linkagem Dinâmica**:

- **Linkagem Dinâmica**: As bibliotecas são carregadas em tempo de execução. O executável depende da presença das bibliotecas corretas no sistema, utilizando mecanismos como `dlopen` para carregar dependências.

- **Linkagem Estática**: As bibliotecas são incorporadas diretamente no executável durante a compilação, resultando em um binário autossuficiente.

**Por que a Linkagem Dinâmica é Necessária neste Projeto**:

- **Uso da Crate `ort` em Rust**:

  - A biblioteca `ort` usada para inferência com ONNX Runtime recomenda o uso da feature `load-dyn`, que carrega o runtime dinamicamente via `dlopen`. Isso torna a linkagem dinâmica obrigatória.

- **Flexibilidade**:

  - Permite que o ONNX Runtime seja atualizado ou substituído sem a necessidade de recompilar o binário.
  - Facilita o uso de diferentes versões do runtime em diversos ambientes.

**Diferenças entre Linkagem Estática e Dinâmica**:

- **Linkagem Estática**:

  - **Vantagens**:
    - Binário autossuficiente.
    - Controle completo do ambiente de execução.
  - **Desvantagens**:
    - Tamanho maior do binário.
    - Não compatível com a feature `load-dyn` da crate `ort`.

- **Linkagem Dinâmica**:

  - **Vantagens**:
    - Binários menores.
    - Atualização de bibliotecas sem recompilação.
    - Necessária para o carregamento dinâmico do ONNX Runtime.
  - **Desvantagens**:
    - Dependência de bibliotecas externas em tempo de execução.

##### 2.4.1 Considerações sobre a Linkagem Dinâmica

A utilização da feature `load-dyn` da crate `ort` exige a linkagem dinâmica para o ONNX Runtime, o que traz flexibilidade, mas requer que o ambiente de execução tenha as bibliotecas corretas disponíveis.

- **Portabilidade via Docker**: A portabilidade pode ser garantida com o uso de contêineres Docker ou ambientes controlados, assegurando que as bibliotecas necessárias estejam presentes.
- **Facilidade de Atualização**: As bibliotecas podem ser atualizadas ou substituídas independentemente do binário, sem a necessidade de recompilação.

**Por que isso é relevante para o projeto?**

- **Necessidade da Crate `ort`**: O uso da feature `load-dyn` torna obrigatória a linkagem dinâmica.
- **Portabilidade**: A utilização de ambientes controlados, como contêineres, garante a portabilidade mesmo com linkagem dinâmica.

#### 2.6 Paralelização do Processamento com Múltiplas Instâncias do Modelo

##### 2.6.1 Motivação para a Paralelização

Inicialmente, o servidor processava as imagens utilizando uma única instância do modelo de detecção de faces. Isso limitava a capacidade de processamento a um único núcleo da CPU, criando um gargalo no desempenho do sistema, especialmente quando várias imagens eram recebidas em um curto espaço de tempo.

Para melhorar a eficiência e aproveitar os recursos multicore das CPUs modernas, implementamos a paralelização do processamento das imagens através da instânciação de múltiplas instâncias do modelo. Isso permite que o servidor processe várias imagens simultaneamente, distribuindo a carga de trabalho entre os núcleos disponíveis e reduzindo o tempo total de processamento.

##### 2.6.2 Implementação de Múltiplas Instâncias do Modelo

No servidor Go, ao iniciar a aplicação, carregamos múltiplas instâncias do modelo de detecção de faces, correspondendo ao número de núcleos da CPU disponíveis menos um (para reservar um núcleo para o sistema operacional e outras tarefas). Cada instância do modelo é independente e pode ser utilizada por uma thread para processar uma imagem.

**Processo de Inicialização:**

- **Identificação dos Núcleos Disponíveis:**
  - Utilizamos a função `runtime.NumCPU()` para determinar o número de núcleos disponíveis na máquina.
  - Subtraímos um desse número para evitar saturar todos os núcleos.

- **Carregamento das Instâncias do Modelo:**
  - Em um loop, carregamos o modelo várias vezes, criando uma instância separada em memória para cada um.
  - As instâncias são armazenadas em uma lista compartilhada (`models`), que é protegida por um mutex (`modelsMutex`) para garantir acesso seguro entre threads.

##### 2.6.3 Gerenciamento de Concorrência com Mutex e Condição de Variável

Para coordenar o acesso às instâncias do modelo entre as diferentes threads que processam as imagens, utilizamos mecanismos de sincronização como mutexes e variáveis de condição (condvars).

- **Mutex (`modelsMutex`):** Garante que apenas uma thread por vez acesse ou modifique a lista de modelos disponíveis. Isso previne condições de corrida ao adicionar ou remover instâncias da lista.

- **Variável de Condição (`modelsCond`):** Permite que as threads esperem até que uma instância do modelo esteja disponível. Se uma thread tentar processar uma imagem e não houver modelos disponíveis, ela aguarda na condição de variável até ser notificada de que uma instância foi liberada.

**Fluxo de Processamento:**

1. **Recepção de Imagens:**
   - Quando uma imagem é recebida via HTTP POST, iniciamos uma goroutine para processá-la, evitando bloquear a recepção de novas imagens.

2. **Acesso ao Modelo:**
   - A goroutine tenta adquirir o `modelsMutex` para acessar a lista de modelos.
   - Se não houver instâncias disponíveis (lista vazia), a goroutine espera na `modelsCond` até ser notificada.

3. **Processamento da Imagem:**
   - Quando uma instância do modelo está disponível, a goroutine a remove da lista e libera o `modelsMutex`.
   - Processa a imagem utilizando o modelo selecionado.
   - Após o processamento, adquire o `modelsMutex` novamente, devolve a instância do modelo à lista e notifica as goroutines esperando na `modelsCond` de que uma instância está disponível.

4. **Notificação:**
   - As goroutines aguardando na `modelsCond` são notificadas sempre que uma instância do modelo é devolvida à lista, permitindo que retomem o processamento.

##### 2.6.4 Benefícios da Abordagem

- **Aumento de Desempenho:**
  - O processamento paralelo das imagens reduz significativamente o tempo de resposta do sistema.
  - Melhor aproveitamento dos recursos da CPU, distribuindo a carga entre múltiplos núcleos.

- **Escalabilidade:**
  - A arquitetura permite ajustar o número de instâncias do modelo de acordo com os recursos disponíveis, facilitando a adaptação a diferentes ambientes de execução.

- **Responsividade:**
  - O uso de goroutines para o processamento assíncrono das imagens evita que a recepção de novas imagens seja bloqueada, aumentando a capacidade de atendimento do servidor.

##### 2.6.5 Sincronização e Segurança de Dados

O uso combinado de mutexes e variáveis de condição garante que:

- **Acesso Seguro aos Modelos:**
  - As instâncias do modelo não são acessadas simultaneamente por múltiplas threads, evitando conflitos e possíveis corrupções de dados.

- **Eficiência na Espera:**
  - As goroutines que aguardam por uma instância do modelo liberada não consomem recursos excessivos, graças ao mecanismo de espera fornecido pelas variáveis de condição.

##### 2.6.6 Considerações sobre o Uso de Recursos

- **Gerenciamento de Memória:**
  - Cada instância do modelo ocupa memória; é importante balancear o número de instâncias com a capacidade de memória do sistema para evitar sobrecarga.

- **Descarte de Modelos:**
  - No encerramento da aplicação, as instâncias do modelo são liberadas corretamente para evitar vazamentos de memória.

##### 2.6.7 Integração com o Fluxo de Imagens

A implementação de múltiplas instâncias do modelo é integrada ao fluxo geral de processamento das imagens:

- **Buffer de Imagens:**
  - As imagens processadas são adicionadas a um buffer que alimenta o stream enviado aos clientes via SSE.

- **Streaming de Imagens:**
  - O servidor continua a enviar as imagens processadas em tempo real aos clientes conectados, aproveitando o aumento de desempenho proporcionado pela paralelização.

#### 2.7 Resumo do Processo em Nível de Aplicação

1. **Construção da Biblioteca Rust**:

   - O código Rust é compilado em uma biblioteca estática (`libyolo_rs.a`).
   - As funções necessárias são expostas via FFI com interface C.

2. **Configuração do CGO no Go**:

   - O Go é instruído a linkar a biblioteca estática do Rust utilizando as diretivas de compilação no cabeçalho do arquivo Go.
   - O cabeçalho C (`bindings.h`) é incluído para que o Go conheça as assinaturas das funções.

3. **Compilação com Linkagem Dinâmica**:

   - As dependências, como o ONNX Runtime, são carregadas em tempo de execução, em vez de estarem incorporadas no binário.

4. **Execução da Aplicação**:

   - O binário requer que o ONNX Runtime esteja disponível no ambiente de execução. O servidor Go inicia, carrega o modelo de detecção de faces e começa a processar as imagens recebidas.

### 3. Frontend: Visualização das Imagens

A visualização das imagens é feita através de uma página [HTML](./frontend) que se conecta ao servidor e recebe o stream de imagens via SSE.

## Como Executar

Clone o projeto incluindo os submodules:

```bash
git clone --recurse-submodules https://github.com/Lemos1347/inteli-modulo-11-ponderada-2.git
```

> [!IMPORTANT]
> Para rodar esse projeto é obrigatório a instalação de [Just](https://github.com/casey/just) e [Docker](https://www.docker.com/) em sua máquina!  
> Para rodar o código embarcado é obrigatório ter em sua máquina [Rust](https://www.rust-lang.org/), ter configurado o [rust hal para esp32](https://docs.esp-rs.org/book/introduction.html) e a biblioteca [esp32-camera](https://github.com/espressif/esp32-camera) baixada (já está sendo incuída caso você tenha clonado o repo com os submodules).

> [!NOTE]
> A seguir estão as dependências de desenvolvimento desse projeto:  
> Para esse projeto é necessario ter configurado em sua máquina: [Rust](https://www.rust-lang.org/) e [Golang](https://go.dev/).   
> Para conseguir utilizar o pacote do embarcado, é necessário seguir as instruções do [rust hal para esp32](https://docs.esp-rs.org/book/introduction.html) e também baixar a biblioteca [esp32-camera](https://github.com/espressif/esp32-camera).  
> É obrigatório seguir as instruções da crate [ort](https://docs.rs/ort/latest/ort/#shared-library-hell) e ter um [onnxruntime](https://github.com/microsoft/onnxruntime/releases) instalado em sua máquina.   
> Para conseguir utilizar a API para dev é crucial que o seu clang esteja devidamente configurado em seu dispositivo.  
> Para alterar as credenciais do wifi e qualquer outra configuração do esp32, crie um arquivo `cfg.toml` com os mesmo campos que em [cfg.toml.example](./embedded/cfg.toml.example)

Após toda a configuração inicial, para rodar tanto o embarcado quando a api há Justfiles.

_API_

```bash
just api
```

_Embarcado_

```bash
just esp32cam
```
