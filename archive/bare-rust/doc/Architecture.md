# Architecture

The design follows a dataflow architecture where multiple tasks
communicate by sending messages to each other. This system is built on
top of BSP (Board Support Package) that interface with the HAL 
(Hardware
Abstraction Layer), which in turn interacts with the hardware.

* Tasks:
  Tasks are individual units of work that are executed periodically.
  Each task has specific information such execution interval, time
  budget, and memory budget.

* Channel:
  Channels are used for inter-task communication. They allow tasks to
  send and receive messages asynchronously.

* Metrics:
  Metrics are used to monitor the performance and resource usage of
  tasks as well as any other metrics in the system as a hole. The
  MetricsTask is responsible for periodically printing these metrics
  to the console and then resetting them.

The following diagram shows the basic architecture of the applications
components.

```mermaid
block-beta
    columns 1
    block: BSP:1
        columns 4
        KeyboardTask
        TextEditTask
        ChatTask
        CryptoTask
        RenderTask
        LinkTask
        MetricsTask
      PttTask
      CodecTask
    end
    block: base:1
        columns 4
        TaskMgr
        Channel
        Metrics
    end
```
## Tasks

### Render 

Render keeps track of all the text and graphics that need to be on 
the screen. The screen is divided up into 20 bands. It then 
periodically renders all the text and graphics into a pixel buffer 
that covers just one band. That buffer is then send to the BSP 
Display module to be displayed on the screen. A band is drawn 
every 10 ms giving a worst case frame rate of 5 fps.The screen is 
240 by 320. The SDI interface supports up to 10 MHz. 

Render has display status icons near the top of the screen and has 
a main text region that is used for displaying text messages as 
well as the text input region. The text input region is used for 
displaying data from the keyboard task as the user types.

The data transferred to the LCD is 16 bit color (5R,6G,5B) and the 
display has a LUT to move it to 18 bit color (6R,6G,6B).  

## Data Flows

The following diagrams show the data flow between tasks for various
operation. The labels on the arrows show the message type used in
the channel for communication.

### Outbound Text Message Data Flow

```mermaid
flowchart LR
    A[Keyboard] -->|Keypress| B[TextEdit]
    B -->|TextInput| C[Chat]
    C -->|TxtMsgOut| D[Crypto]
    B -->|print_input| J[Render]
    D -->|EncTxtMsgOut| F[Link]
    C -->|print| J[Render]
    C -->|print_clear| J[Render]
```

### Inbound Text Message Data Flow

```mermaid
flowchart LR
    A[Link] -->|EncTxtMsgIn| B[Crypto]
    B -->|TxtMsgIn| C[Chat]
    C -->|print| J[Render]
```

### Outbound Audio Data Flow

```mermaid
flowchart LR
    A[Audio] -->|AudOut| B[Codec]
    B -->|EncAudOut| C[PTT]
    C -->|AudMsgOut| D[Crypto]
    D -->|EncAudMsgOut| F[Link]
```

### Inbound Audio Data Flow

```mermaid
flowchart LR
    A[Link] -->|EncAudMsgIn| B[Crypto]
    B -->|AudMsgIn| C[PTT]
    C -->|EncAudIn| J[Codec]
    J -->|AudIn| K[Audio]
```

### Group Selection Data Flow

TODO - describe how the group selection works

### Remote Storage Data Flows

TODO - describe flows for remote storage

# Board Support and Hardware Abstraction Layers

The following diagram shows how the applications uses modules from the
BSP layer which in turn use the HAL layer to interact with the
hardware.

```mermaid
block-beta
    columns 1
    Application
    block: BSP
      columns 7
      Display
      Keyboard
      Buttons
      Info
      Audio
      Console
      Link
      Led
      Debug
      Battery
    end
    space
    block: HAL
      SPI
      GPIO
      ADC
      I2C
      I2S
      UART
    end
    block: Hardware
      ILI9341 ("LCD\nILI9341")
      Switches
      LED ("LED\nRGB")
      M24C02 ("EEPROM\nMC24C02")
      WM8960 ("Audio\nM8960")
      CH340 ("USB\nCH340")
      NetCPU ("NetCPU\nESP32")
    end
    Keyboard --> GPIO
    Display --> SPI
    Display --> GPIO
    Led --> GPIO
    Debug --> GPIO
    Audio --> I2S
    Audio --> I2C
    Info --> I2C
    Battery --> ADC
    Console --> UART
    Buttons --> GPIO
    Link --> UART
    SPI --> ILI9341
    UART --> NetCPU
    I2C --> WM8960
    I2S --> WM8960
    I2C --> M24C02
    GPIO --> Switches
    GPIO --> LED
    UART --> CH340
```

