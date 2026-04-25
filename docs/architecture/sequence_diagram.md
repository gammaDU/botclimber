# Botclimber: User Interaction Sequence Diagrams

This document illustrates the two distinct interaction cycles within the Botclimber system: **Account Management** (performed by a human via the Web Frontend) and **Bot Gameplay** (automated via the Bot API).

## 1. Account Management Cycle (Human)

This phase occurs through the Web Frontend and involves creating an account, authenticating, and managing bot credentials.

```mermaid
sequenceDiagram
    autonumber
    actor P as Player (User)
    participant F as Web Frontend
    participant G as API Gateway
    participant A as Auth Service
    participant DB as Persistence Layer

    Note over P, DB: Registration & Login
    P->>F: Open Registration Page
    P->>F: Enter Username/Password
    F->>G: POST /api/v1/user/register
    G->>A: Create Account
    A->>DB: Save User Credentials
    DB-->>A: Success
    A-->>G: Created
    G-->>F: 201 Created
    
    P->>F: Login
    F->>G: POST /api/v1/user/login
    G->>A: Authenticate
    A->>DB: Verify Credentials
    DB-->>A: Valid
    A-->>G: JWT Token
    G-->>F: 200 OK (Token)

    Note over P, DB: Bot Credential Management
    P->>F: Generate API Key
    F->>G: POST /api/v1/user/apikey (with JWT)
    G->>A: Generate Key
    A->>DB: Save Key Mapping
    DB-->>A: Success
    A-->>G: API Key
    G-->>F: 200 OK (API Key)
    F-->>P: Display API Key
```

## 2. Bot Gameplay Cycle (Automated)

This phase is performed by the user's Bot Client using the unique API Key generated in Phase 1. It is technically independent of the Web Frontend.

```mermaid
sequenceDiagram
    autonumber
    actor B as Bot Client
    participant G as API Gateway
    participant A as Auth Service
    participant S as Session Manager
    participant E as Physics Engine
    participant DB as Persistence Layer

    Note over B, DB: Session Initialization
    B->>G: POST /api/v1/bot/join (with X-API-Key)
    G->>A: Validate API Key
    A-->>G: Valid (Player ID)
    G->>S: Join Game
    S->>E: Spawn Player Figure
    S-->>G: Player Details
    G-->>B: 200 OK (Player ID, Spawn Point)

    loop Gameplay Loop
        loop Initial State Check
            B->>G: GET /api/v1/bot/state
            G->>S: Get Current State
            S->>E: Query Position/Platforms/State
            E-->>S: Spatial Data
            S-->>G: Game State
            G-->>B: 200 OK (Position, Platforms, State)
            Note over B: Ensure state == LANDED
        end

        B->>B: Calculate Jump
        
        B->>G: POST /api/v1/bot/jump (Strength, Direction)
        G->>S: Execute Jump
        S->>E: Apply Impulse
        E-->>S: Success
        S-->>G: Jump Accepted
        G-->>B: 200 OK
        
        loop Polling State
            B->>G: GET /api/v1/bot/state
            G->>S: Get Current State
            S->>E: Query Position/State
            E-->>S: Spatial Data
            S-->>G: Game State (Position, State)
            G-->>B: 200 OK (e.g., state: JUMPING)
            Note over B: Bot waits until state == LANDED
        end
        
        Note right of E: Physics Engine processes<br/>movement & reflections (60Hz)
        E->>DB: Update Max Height (if record)
    end

    Note over B, DB: Session Termination
    B->>G: POST /api/v1/bot/leave
    G->>S: End Session
    S->>E: Remove Figure
    S-->>G: Success
    G-->>B: 204 No Content
```

## Phase Descriptions

### 1. Account Management
This cycle is typically performed by a human user through the **Web Frontend**.
*   **Registration & Login**: The user creates an account and authenticates to receive a JWT token.
*   **API Key Generation**: The user retrieves a unique API key, which is the secret used by their bot for all gameplay interactions.

### 2. Bot Gameplay
This cycle is performed by the user's **Bot Client** using the Bot API.
*   **Joining**: The bot enters the game world. The server associates the API key with a unique player session.
*   **The Loop**:
    *   **State Retrieval**: The bot polls the spatial state to understand its surroundings (platforms and position).
    *   **Decision Making**: The bot calculates the optimal jump based on the **Game Rules**.
    *   **Jump Command**: The bot sends the jump vector to the server.
*   **Physics Simulation**: The server's Physics Engine handles the actual movement, gravity, and reflections asynchronously at 60Hz.
*   **Graceful Exit**: The bot informs the server when it is finished to free up resources.
