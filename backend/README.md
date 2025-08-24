# TwoDo Backend

A production-ready Rust backend built with Axum for couples to manage their daily tasks, chores, expenses, and shared calendar.

## 🚀 Features

### Authentication Service
- **User Registration**: Secure user registration with Argon2 password hashing
- **User Login**: JWT-based authentication with configurable expiration
- **Input Validation**: Username (3-50 chars) and password (8+ chars) validation

### Chores Management
- **Create Chores**: Assign chores with deadlines, categories, and recurrence
- **Categories**: Custom categorization with colors
- **Recurring Tasks**: Support for daily, weekly, monthly recurring chores
- **Group Management**: Share chores within family/couple groups

### Expense Tracking (Splitwise-like)
- **Expense Creation**: Add expenses with custom splits
- **Balance Calculation**: Automatic balance tracking between users
- **Group Expenses**: Manage shared expenses within groups
- **Split Management**: Flexible expense splitting between users

### Shared Calendar
- **Calendar Creation**: Create shared calendars for groups
- **Event Management**: Add events with start/end times
- **Multi-calendar Support**: Support for multiple calendars per group

## 🛠 Technology Stack

- **Language**: Rust 🦀
- **Web Framework**: Axum 0.7
- **Architecture**: Hexagonal Architecture (Ports & Adapters)
- **Authentication**: JWT with Argon2 password hashing
- **Validation**: Input validation with the `validator` crate
- **Logging**: Structured logging with `tracing`
- **CORS**: Configured for frontend communication
- **Data Storage**: In-memory repositories (ready for database integration)

## 📁 Project Structure

```
src/
├── main.rs                     # Application entry point
├── config.rs                   # Configuration management
├── auth/                       # Authentication module
│   ├── domain/                 # Domain entities (User)
│   ├── application/            # Business logic (AuthService)
│   └── infrastructure/         # Web routes & persistence
├── chores/                     # Chores management module
│   ├── domain/                 # Domain entities (Chore, Category)
│   ├── application/            # Business logic (ChoreService)
│   └── infrastructure/         # Web routes & persistence
├── expenses/                   # Expense tracking module
│   ├── domain/                 # Domain entities (Expense, Split)
│   ├── application/            # Business logic (ExpenseService)
│   └── infrastructure/         # Web routes & persistence
└── calendar/                   # Calendar management module
    ├── domain/                 # Domain entities (Calendar, Event)
    ├── application/            # Business logic (CalendarService)
    └── infrastructure/         # Web routes & persistence
```

## 🔧 Prerequisites

- **Rust**: 1.70 or later
- **Cargo**: Latest version

## 🚀 Running Locally

### 1. Clone and Navigate
```bash
cd twodo/backend
```

### 2. Set Environment Variables (Optional)
The application uses sensible defaults, but you can configure:

```bash
export HOST=127.0.0.1
export PORT=3000
export JWT_SECRET=your-super-secure-jwt-secret-key-here-min-256-bits
export JWT_EXPIRATION_HOURS=24
export RUST_LOG=debug
```

### 3. Build and Run
```bash
cargo build
cargo run
```

The server will start on `http://127.0.0.1:3000`

## 📝 API Documentation

### Base URL
```
http://127.0.0.1:3000
```

### Authentication Endpoints

#### Register User
```bash
POST /api/auth/register
Content-Type: application/json

{
  "username": "john_doe",
  "password": "securepassword123"
}
```

**Response (200)**:
```json
{
  "id": "uuid-here",
  "username": "john_doe"
}
```

#### Login User
```bash
POST /api/auth/login
Content-Type: application/json

{
  "username": "john_doe",
  "password": "securepassword123"
}
```

**Response (200)**:
```json
{
  "token": "jwt-token-here"
}
```

### Chores Endpoints

#### Create Chore
```bash
POST /api/chores
Content-Type: application/json

{
  "title": "Take out trash",
  "description": "Weekly garbage pickup",
  "assigned_to": "user-uuid",
  "deadline": "2025-08-25T10:00:00Z",
  "category": {"name": "Household", "color": "#FF5733"},
  "recurring": "Weekly",
  "group_id": "group-uuid"
}
```

### Expenses Endpoints

#### Create Expense
```bash
POST /api/expenses
Content-Type: application/json

{
  "description": "Grocery shopping",
  "amount": 85.50,
  "paid_by": "user-uuid",
  "group_id": "group-uuid",
  "splits": [
    {"user_id": "user1-uuid", "amount": 42.75},
    {"user_id": "user2-uuid", "amount": 42.75}
  ]
}
```

#### Get Balances
```bash
GET /api/expenses/balances/{group_id}
```

### Calendar Endpoints

#### Create Calendar
```bash
POST /api/calendar
Content-Type: application/json

{
  "name": "Family Calendar",
  "group_id": "group-uuid"
}
```

#### Create Event
```bash
POST /api/calendar/{calendar_id}/events
Content-Type: application/json

{
  "title": "Doctor Appointment",
  "description": "Annual checkup",
  "start_time": "2025-08-25T14:00:00Z",
  "end_time": "2025-08-25T15:00:00Z"
}
```

## 🧪 Testing

### Manual API Testing

1. **Start the server**:
   ```bash
   cargo run
   ```

2. **Test authentication**:
   ```bash
   # Register a user
   curl -X POST http://127.0.0.1:3000/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{"username": "testuser", "password": "testpass123"}'
   
   # Login
   curl -X POST http://127.0.0.1:3000/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username": "testuser", "password": "testpass123"}'
   ```

3. **Test validation** (should return 400):
   ```bash
   curl -X POST http://127.0.0.1:3000/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{"username": "ab", "password": "short"}'
   ```

## 🔒 Security Features

- **Password Hashing**: Argon2 for secure password storage
- **JWT Authentication**: Configurable token expiration
- **Input Validation**: Comprehensive validation on all endpoints
- **CORS Configuration**: Properly configured for cross-origin requests
- **Environment Configuration**: Secrets managed via environment variables

## 🚧 Production Considerations

### Database Integration
Currently using in-memory repositories. For production:

1. **Add database dependencies**:
   ```toml
   [dependencies]
   sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }
   ```

2. **Implement database repositories**:
   - Replace `InMemory*Repository` with database implementations
   - Add migrations for schema management

### Security Enhancements
- Add rate limiting
- Implement API key authentication for service-to-service calls
- Add request/response logging
- Implement audit logging

### Monitoring & Observability
- Add health check endpoints
- Implement metrics collection (Prometheus)
- Add distributed tracing
- Error reporting integration

## 📊 Performance

- **Architecture**: Modular monolith for easy deployment and development
- **Async/Await**: Full async support with Tokio runtime
- **Memory Usage**: Efficient memory usage with Rust's ownership model
- **Concurrency**: Built for high-concurrency workloads

## 🤝 Contributing

1. Follow hexagonal architecture patterns
2. Add tests for new functionality
3. Validate all inputs
4. Update documentation
5. Ensure production-ready code quality

---

**Built with ❤️ in Rust**
