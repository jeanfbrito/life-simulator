# Life Simulator - Excalibur Viewer

A modern ExcaliburJS-based frontend for the Life Simulator project, providing an isometric view of the living world with enhanced performance and features.

## Features

- **Isometric Rendering**: Beautiful 2.5D isometric view using ExcaliburJS
- **Real-time Updates**: HTTP polling for live entity positions and world state
- **Performance Monitoring**: Built-in FPS counter and performance metrics
- **Interactive Controls**: Pan, zoom, and navigate the world seamlessly
- **Modular Architecture**: Clean separation of concerns with TypeScript
- **Comprehensive Testing**: Playwright e2e tests and Vitest unit tests

## Development

### Prerequisites

- Node.js 18+
- npm or pnpm

### Getting Started

1. **Install dependencies**:
   ```bash
   npm install
   ```

2. **Start development server**:
   ```bash
   npm run dev
   ```

3. **Open in browser**:
   Navigate to `http://localhost:3000`

### Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run test` - Run unit tests
- `npm run test:ui` - Run unit tests with UI
- `npm run test:e2e` - Run end-to-end tests
- `npm run test:e2e:ui` - Run e2e tests with UI
- `npm run lint` - Run ESLint
- `npm run lint:fix` - Fix ESLint issues
- `npm run format` - Format code with Prettier
- `npm run type-check` - Check TypeScript types

### Architecture

```
src/
├── engine/          # Core Excalibur engine setup
├── data/           # API client and data management
├── terrain/        # Terrain rendering system
├── entities/       # Entity management and rendering
├── overlays/       # UI overlays and visualizations
├── camera/         # Camera controls and input handling
├── utils/          # Utility functions and helpers
└── main.ts         # Application entry point
```

### Backend Integration

The viewer connects to the Life Simulator backend running on `http://127.0.0.1:54321` with the following API endpoints:

- `/api/world_info` - Current world metadata
- `/api/chunks` - Terrain and resource data
- `/api/entities` - Entity positions and states
- `/api/species` - Species configuration
- `/api/vegetation/metrics` - Biomass data

## Testing

### Unit Tests
```bash
npm run test
```

### End-to-End Tests
```bash
npm run test:e2e
```

### Visual Testing
The project includes Playwright tests with screenshot capabilities for visual regression testing.

## Configuration

### Environment Variables

- `VITE_API_BASE_URL` - Backend API URL (default: `http://127.0.0.1:54321`)

### Development Settings

Development settings are configured in `vite.config.ts` including proxy configuration for API requests.

## Contributing

1. Follow the existing code style (ESLint + Prettier)
2. Add tests for new features
3. Update documentation as needed
4. Ensure all tests pass before submitting

## License

MIT License - see LICENSE file for details.