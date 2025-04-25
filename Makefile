all1:
	cargo build --release
	mv target/release/rust_spreadsheet target/release/spreadsheet

clean1:
	cargo clean

# Variables
BACKEND_DIR = backend
FRONTEND_DIR = frontend
BACKEND_BINARY = target/release/spreadsheet
PORT = 8080
URL = http://localhost:$(PORT)

# Default target: build everything
all: build

# Build both backend and frontend
build: build-backend build-frontend

# Build the backend in release mode
build-backend:
	@echo "Building backend..."
	cd $(BACKEND_DIR) && cargo build --release
	@if [ -f $(BACKEND_DIR)/target/release/rust_spreadsheet ] && [ ! -f target/release/spreadsheet ]; then \
		mkdir -p target/release && \
		cp $(BACKEND_DIR)/target/release/rust_spreadsheet $(BACKEND_BINARY); \
	elif [ -f $(BACKEND_DIR)/target/release/spreadsheet ] && [ ! -f target/release/spreadsheet ]; then \
		mkdir -p target/release && \
		cp $(BACKEND_DIR)/target/release/spreadsheet $(BACKEND_BINARY); \
	fi

# Build the frontend using trunk
build-frontend:
	@echo "Building frontend..."
	cd $(FRONTEND_DIR) && trunk build --release

# Run the application (both backend and frontend)
run: build
	@echo "Starting application..."
	@$(MAKE) run-backend & 
	@sleep 2
	@$(MAKE) run-frontend-dev
	@echo "Press Ctrl+C to stop the server"
	@wait

# Run the backend server
# Run the backend server
run-backend:
	@echo "Checking if backend server is already running on $(URL)..."
	@if curl -s --head $(URL) > /dev/null 2>&1; then \
		echo "🔄 Backend is already running at $(URL)"; \
	else \
		echo "🚀 Starting backend server on $(URL)..."; \
		cd $(BACKEND_DIR) && cargo run --release; \
	fi

# Run the frontend development server
run-frontend-dev:
	@echo "Starting frontend development server..."
	@cd $(FRONTEND_DIR) && trunk serve

# Open the web browser
open-browser:
	@echo "Opening application in browser..."
	@xdg-open $(URL) || open $(URL) || start $(URL) || echo "Could not open browser automatically"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@rm -rf $(FRONTEND_DIR)/dist
	cd frontend && cargo clean
	cd backend && cargo clean
	rm -rf main.pdf

# Development mode: backend in release mode, frontend with hot reloading
dev:
	@echo "Starting backend in one terminal..."
	@$(MAKE) build-backend
	@$(MAKE) run-backend &
	@echo "Starting frontend development server..."
	@cd $(FRONTEND_DIR) && trunk serve --open
	@wait

docs:
	cd report && pdflatex main.tex
	mv report/main.pdf main.pdf
	cargo doc 
	cd frontend && cargo doc 
	cd backend && cargo doc



.PHONY: all build build-backend build-frontend run run-backend run-frontend-dev open-browser clean dev