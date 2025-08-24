# TwoDo: A Shared Management App for Couples

TwoDo is a comprehensive application designed to help couples and families manage their daily lives. From splitting expenses to organizing chores and scheduling events, TwoDo aims to be the central hub for a shared life.

## 📁 Project Structure

```
twodo/
├── docs/                    # 📚 Documentation
│   ├── backend/            # Backend API docs
│   ├── deployment/         # Deployment guides
│   └── security/           # Security setup
├── scripts/                # 🔧 Automation scripts
│   ├── backend/            # Backend utilities
│   └── deployment/         # Deployment scripts
├── backend/                # 🦀 Rust Backend (Cloudflare Workers)
├── frontend/               # ⚛️ React Native App
└── deployment/             # ☁️ Infrastructure configs
```

## 📖 Documentation

📄 **[Complete Documentation](docs/README.md)** - Detailed guides and setup instructions

## Features

- **Chore Management**: Create, assign, and track household chores. Set deadlines, categorize tasks, and set up recurring chores.
- **Expense Splitting**: A "Splitwise-like" feature to easily split bills and track shared expenses.
- **Shared Calendar**: A collaborative calendar to manage events, set reminders, and keep track of important dates.
- **User-Specific To-Do Lists**: Each user can maintain their own daily to-do list, with options for recurring tasks.
- **Secure Authentication**: All routes are secured, ensuring that user data is safe and private.

## Technology Stack

This project is built with a modern, robust technology stack, emphasizing performance, scalability, and maintainability.

### Backend

- **Language**: Rust
- **Platform**: Cloudflare Workers (WASM)
- **Architecture**: Hexagonal Architecture (Ports and Adapters)
- **Database**: Cloudflare D1 (SQLite)
- **Storage**: Cloudflare R2
- **Authentication**: JWT + WASM-compatible crypto
- **Status**: ✅ Authentication APIs fully functional

### Frontend

- **Framework**: React Native (for iOS)

## Getting Started

### Prerequisites

- Rust and Cargo
- Node.js and npm/yarn
- An iOS simulator (or a physical device)

### Quick Start

1. **Backend Setup**: Follow [Quick Setup Guide](docs/deployment/QUICK_SETUP.md)
2. **Testing APIs**: Use [API Testing Guide](docs/backend/API_TESTING.md)
3. **Security**: Configure with [Security Setup](docs/security/SECURITY_SETUP.md)

### Development

**Backend (Cloudflare Workers)**:
```bash
cd backend
worker-build --release  # Build for WASM
wrangler deploy         # Deploy to Cloudflare
```

**Frontend (React Native)**:
```bash
cd frontend
npm install
npx react-native run-ios
```

### Testing

Each service will have its own set of tests. To run the tests for a service:

```bash
cd backend/choresService
cargo test
```

API endpoints can also be tested using tools like `curl` or Postman. For example, to create a new chore:

```bash
curl -X POST \
  http://localhost:3000/chores \
  -H 'Content-Type: application/json' \
  -d '{
    "title": "Clean the kitchen",
    "description": "Wipe the counters, do the dishes, and take out the trash",
    "assigned_to": "a1b2c3d4-e5f6-a7b8-c9d0-e1f2a3b4c5d6",
    "category": {
      "id": "f0e9d8c7-b6a5-f4e3-d2c1-b0a9f8e7d6c5",
      "name": "Cleaning"
    },
    "group_id": "b1a2c3d4-e5f6-a7b8-c9d0-e1f2a3b4c5d6"
  }'
```
