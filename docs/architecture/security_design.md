# Botclimber: Security Design

This document outlines the security strategy for the Botclimber server, focusing on protecting system resources, ensuring fair play, and managing access control.

## 1. Threat Model Summary
The primary security concerns for Botclimber are:
- **Resource Exhaustion**: Accidental or intentional flooding of the API by bot developers, leading to server instability.
- **Unauthorized Access**: Manipulation of other players' figures or accessing administrative functions without permission.
- **Impersonation**: Using a stolen API key or JWT to act as another user.

## 2. Authentication Mechanisms

### 2.1 Bot API (X-API-Key)
- **Scope**: Used for gameplay commands (`Join`, `Jump`, `State`, `Leave`).
- **Implementation**: A unique, high-entropy string generated per user.
- **Requirement**: Must be sent in the `X-API-Key` header of every request to the `/api/v1/bot/` namespace.
- **Rotation**: Users can regenerate their API key via the web frontend at any time.

### 2.2 Frontend and Admin API (JWT)
- **Scope**: Used for account management and administrative tasks.
- **Implementation**: JSON Web Tokens (JWT) signed using a server-side secret (HMAC SHA-256).
- **Workflow**: 
    1. User logs in with username/password via `POST /api/v1/user/login`.
    2. Server returns a Bearer token.
    3. Client includes token in `Authorization: Bearer <token>` header for subsequent requests.
- **Expiration**: Short-lived tokens to minimize the impact of token theft.

## 3. Rate Limiting and Flood Protection
As defined in **US08**, the system implements a multi-tiered rate limiting strategy using the **Token Bucket** algorithm.

### 3.1 Tiers
1.  **Global IP-based Limit**: 
    - Applies to all unauthenticated requests.
    - Prevents basic DoS attacks and brute-force attempts on the login/registration endpoints.
2.  **Bot-specific Limit (Per API Key)**:
    - Applies to gameplay commands.
    - Allows for short bursts (e.g., rapid state polling) while maintaining a sustainable long-term rate.
3.  **Frontend Limit (Per User)**:
    - Limits frequency of account updates and API key regenerations.

### 3.2 Token Bucket Configuration
- **Limit**: The average number of requests allowed per second.
- **Burst**: The maximum number of tokens the bucket can hold at once.
- **Header Feedback**:
    - `RateLimit-Limit`: Maximum requests allowed in the window.
    - `RateLimit-Remaining`: Tokens left in the bucket.
    - `RateLimit-Reset`: Time until the bucket is full again.

## 4. Authorization and Scoping
- **Principle of Least Privilege**: Bots can only control the figure associated with their API key.
- **Namespace Separation**: 
    - `/bot/`: Accessible via API Key.
    - `/user/`: Accessible via User JWT.
    - `/admin/`: Accessible only via User JWT with the `is_admin` flag set to `true`.

## 5. Security Recommendations for Bot Developers
- **Secure Key Storage**: Do not hardcode API keys in public repositories. Use environment variables.
- **Error Handling**: Respect `429 Too Many Requests` responses and the `Retry-After` header to avoid being temporarily blacklisted.
- **State Polling**: Wait for the `LANDED` state before sending the next `Jump` command to stay within rate limits.
