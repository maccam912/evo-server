// Renderer state
const RendererState = {
    canvas: null,
    ctx: null,
    worldWidth: 0,
    worldHeight: 0,

    // View state
    offsetX: 0,
    offsetY: 0,
    scale: 1,
    cellSize: 8, // Base size of each grid cell

    // Mouse state
    isDragging: false,
    lastMouseX: 0,
    lastMouseY: 0,

    // Touch state
    isTouching: false,
    lastTouchX: 0,
    lastTouchY: 0,
};

// Initialize the renderer
function initializeRenderer(worldWidth, worldHeight) {
    RendererState.canvas = document.getElementById('world-canvas');
    RendererState.ctx = RendererState.canvas.getContext('2d');
    RendererState.worldWidth = worldWidth;
    RendererState.worldHeight = worldHeight;

    // Set canvas size to fit container
    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);

    // Center the view
    centerView();

    // Set up event listeners
    setupEventListeners();

    console.log('Renderer initialized');
}

// Resize canvas to fit container
function resizeCanvas() {
    const container = RendererState.canvas.parentElement;
    RendererState.canvas.width = container.clientWidth;
    RendererState.canvas.height = container.clientHeight;
}

// Center the view on the world
function centerView() {
    const canvasWidth = RendererState.canvas.width;
    const canvasHeight = RendererState.canvas.height;
    const worldPixelWidth = RendererState.worldWidth * RendererState.cellSize * RendererState.scale;
    const worldPixelHeight = RendererState.worldHeight * RendererState.cellSize * RendererState.scale;

    RendererState.offsetX = (canvasWidth - worldPixelWidth) / 2;
    RendererState.offsetY = (canvasHeight - worldPixelHeight) / 2;
}

// Set up event listeners
function setupEventListeners() {
    const canvas = RendererState.canvas;

    // Mouse drag for panning
    canvas.addEventListener('mousedown', (e) => {
        RendererState.isDragging = true;
        RendererState.lastMouseX = e.clientX;
        RendererState.lastMouseY = e.clientY;
        canvas.style.cursor = 'grabbing';
    });

    canvas.addEventListener('mousemove', (e) => {
        if (RendererState.isDragging) {
            const dx = e.clientX - RendererState.lastMouseX;
            const dy = e.clientY - RendererState.lastMouseY;
            RendererState.offsetX += dx;
            RendererState.offsetY += dy;
            RendererState.lastMouseX = e.clientX;
            RendererState.lastMouseY = e.clientY;
        }
    });

    canvas.addEventListener('mouseup', () => {
        RendererState.isDragging = false;
        canvas.style.cursor = 'crosshair';
    });

    canvas.addEventListener('mouseleave', () => {
        RendererState.isDragging = false;
        canvas.style.cursor = 'crosshair';
    });

    // Mouse click for creature selection
    canvas.addEventListener('click', (e) => {
        if (!RendererState.isDragging) {
            handleCanvasClick(e);
        }
    });

    // Mouse wheel for zooming
    canvas.addEventListener('wheel', (e) => {
        e.preventDefault();
        const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
        const mouseX = e.clientX - canvas.offsetLeft;
        const mouseY = e.clientY - canvas.offsetTop;

        // Zoom towards mouse position
        RendererState.offsetX = mouseX - (mouseX - RendererState.offsetX) * zoomFactor;
        RendererState.offsetY = mouseY - (mouseY - RendererState.offsetY) * zoomFactor;
        RendererState.scale *= zoomFactor;

        // Clamp scale
        RendererState.scale = Math.max(0.1, Math.min(10, RendererState.scale));
    });

    // Touch events for mobile panning
    canvas.addEventListener('touchstart', (e) => {
        if (e.touches.length === 1) {
            e.preventDefault();
            RendererState.isTouching = true;
            const touch = e.touches[0];
            RendererState.lastTouchX = touch.clientX;
            RendererState.lastTouchY = touch.clientY;
        }
    }, { passive: false });

    canvas.addEventListener('touchmove', (e) => {
        if (RendererState.isTouching && e.touches.length === 1) {
            e.preventDefault();
            const touch = e.touches[0];
            const dx = touch.clientX - RendererState.lastTouchX;
            const dy = touch.clientY - RendererState.lastTouchY;
            RendererState.offsetX += dx;
            RendererState.offsetY += dy;
            RendererState.lastTouchX = touch.clientX;
            RendererState.lastTouchY = touch.clientY;
        }
    }, { passive: false });

    canvas.addEventListener('touchend', (e) => {
        if (e.touches.length === 0) {
            RendererState.isTouching = false;
        }
    });

    canvas.addEventListener('touchcancel', () => {
        RendererState.isTouching = false;
    });

    // Zoom controls
    document.getElementById('zoom-in').addEventListener('click', () => {
        RendererState.scale *= 1.2;
        RendererState.scale = Math.min(10, RendererState.scale);
    });

    document.getElementById('zoom-out').addEventListener('click', () => {
        RendererState.scale *= 0.8;
        RendererState.scale = Math.max(0.1, RendererState.scale);
    });

    document.getElementById('reset-view').addEventListener('click', () => {
        RendererState.scale = 1;
        centerView();
    });
}

// Handle canvas click for creature selection
function handleCanvasClick(e) {
    const rect = RendererState.canvas.getBoundingClientRect();
    const canvasX = e.clientX - rect.left;
    const canvasY = e.clientY - rect.top;

    // Convert canvas coordinates to world coordinates
    const worldX = (canvasX - RendererState.offsetX) / (RendererState.cellSize * RendererState.scale);
    const worldY = (canvasY - RendererState.offsetY) / (RendererState.cellSize * RendererState.scale);

    // Find creature at this position
    const clickedCreature = findCreatureAtPosition(worldX, worldY);

    if (clickedCreature) {
        window.selectCreature(clickedCreature);
    } else {
        window.deselectCreature();
    }
}

// Find creature at world position
function findCreatureAtPosition(worldX, worldY) {
    const appState = window.AppState;
    if (!appState || !appState.creatures) return null;

    const threshold = 2; // Click threshold in world units

    for (const creature of appState.creatures) {
        const dx = creature.x - worldX;
        const dy = creature.y - worldY;
        const distance = Math.sqrt(dx * dx + dy * dy);

        if (distance < threshold) {
            return creature;
        }
    }

    return null;
}

// Render the world
function renderWorld(appState) {
    const ctx = RendererState.ctx;
    if (!ctx) return;

    const canvas = RendererState.canvas;
    const cellSize = RendererState.cellSize * RendererState.scale;

    // Clear canvas
    ctx.fillStyle = '#0a0a0a';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Save context state
    ctx.save();

    // Apply transformations
    ctx.translate(RendererState.offsetX, RendererState.offsetY);
    ctx.scale(RendererState.scale, RendererState.scale);

    // Draw grid
    drawGrid(ctx);

    // Draw creatures
    if (appState.creatures && appState.creatures.length > 0) {
        drawCreatures(ctx, appState.creatures);
    }

    // Restore context state
    ctx.restore();
}

// Draw grid
function drawGrid(ctx) {
    const cellSize = RendererState.cellSize;
    const width = RendererState.worldWidth;
    const height = RendererState.worldHeight;

    ctx.strokeStyle = '#1a1a1a';
    ctx.lineWidth = 0.5 / RendererState.scale;

    // Draw vertical lines
    for (let x = 0; x <= width; x += 10) {
        ctx.beginPath();
        ctx.moveTo(x * cellSize, 0);
        ctx.lineTo(x * cellSize, height * cellSize);
        ctx.stroke();
    }

    // Draw horizontal lines
    for (let y = 0; y <= height; y += 10) {
        ctx.beginPath();
        ctx.moveTo(0, y * cellSize);
        ctx.lineTo(width * cellSize, y * cellSize);
        ctx.stroke();
    }

    // Draw border
    ctx.strokeStyle = '#3a3a3a';
    ctx.lineWidth = 2 / RendererState.scale;
    ctx.strokeRect(0, 0, width * cellSize, height * cellSize);
}

// Draw creatures
function drawCreatures(ctx, creatures) {
    const cellSize = RendererState.cellSize;

    creatures.forEach(creature => {
        // Calculate color based on energy (0-100 range assumed)
        const energy = creature.energy || 0;
        const hue = Math.min(120, (energy / 100) * 120); // 0 (red) to 120 (green)
        const color = `hsl(${hue}, 80%, 50%)`;

        // Draw creature as a circle
        ctx.fillStyle = color;
        ctx.beginPath();
        ctx.arc(
            creature.x * cellSize,
            creature.y * cellSize,
            cellSize * 0.6, // Radius
            0,
            2 * Math.PI
        );
        ctx.fill();

        // Highlight selected creature
        if (window.AppState && window.AppState.selectedCreature &&
            window.AppState.selectedCreature.id === creature.id) {
            ctx.strokeStyle = '#ffeb3b';
            ctx.lineWidth = 2 / RendererState.scale;
            ctx.stroke();
        }
    });
}

// Export functions
window.initializeRenderer = initializeRenderer;
window.renderWorld = renderWorld;
