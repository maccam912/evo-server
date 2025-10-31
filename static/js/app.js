// Application state
const AppState = {
    ws: null,
    connected: false,
    reconnectAttempts: 0,
    maxReconnectAttempts: 10,
    reconnectDelay: 2000,

    // Simulation state
    worldWidth: 0,
    worldHeight: 0,
    creatures: [],
    metrics: {},

    // UI state
    selectedCreature: null,
};

// WebSocket connection
function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    updateConnectionStatus('connecting');
    console.log(`Connecting to WebSocket: ${wsUrl}`);

    AppState.ws = new WebSocket(wsUrl);

    AppState.ws.onopen = () => {
        console.log('WebSocket connected');
        AppState.connected = true;
        AppState.reconnectAttempts = 0;
        updateConnectionStatus('connected');

        // Request initial state
        sendMessage({ type: 'get_state' });
    };

    AppState.ws.onmessage = (event) => {
        try {
            const message = JSON.parse(event.data);
            handleServerMessage(message);
        } catch (error) {
            console.error('Failed to parse message:', error);
        }
    };

    AppState.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };

    AppState.ws.onclose = () => {
        console.log('WebSocket closed');
        AppState.connected = false;
        updateConnectionStatus('disconnected');

        // Attempt to reconnect
        if (AppState.reconnectAttempts < AppState.maxReconnectAttempts) {
            AppState.reconnectAttempts++;
            console.log(`Reconnecting... (attempt ${AppState.reconnectAttempts}/${AppState.maxReconnectAttempts})`);
            setTimeout(connectWebSocket, AppState.reconnectDelay);
        } else {
            console.error('Max reconnection attempts reached');
            updateConnectionStatus('disconnected');
        }
    };
}

// Send message to server
function sendMessage(message) {
    if (AppState.ws && AppState.ws.readyState === WebSocket.OPEN) {
        AppState.ws.send(JSON.stringify(message));
    } else {
        console.warn('WebSocket not connected, cannot send message');
    }
}

// Handle messages from server
function handleServerMessage(message) {
    switch (message.type) {
        case 'update':
            handleUpdate(message);
            break;
        case 'full_state':
            handleFullState(message);
            break;
        case 'world_region':
            console.log('Received world region (not implemented)');
            break;
        default:
            console.warn('Unknown message type:', message.type);
    }
}

// Handle update message
function handleUpdate(message) {
    AppState.metrics = message.metrics || {};
    AppState.creatures = message.creatures || [];

    // Update UI
    if (window.updateUI) {
        window.updateUI(AppState);
    }

    // Render the world
    if (window.renderWorld) {
        window.renderWorld(AppState);
    }
}

// Handle full state message
function handleFullState(message) {
    AppState.metrics = message.metrics || {};
    AppState.creatures = message.creatures || [];

    // Extract world dimensions (sent directly in message, not nested in world object)
    AppState.worldWidth = message.world_width || 0;
    AppState.worldHeight = message.world_height || 0;

    console.log(`World size: ${AppState.worldWidth}x${AppState.worldHeight}`);
    console.log(`Creatures: ${AppState.creatures.length}`);

    // Initialize renderer with world dimensions
    if (window.initializeRenderer) {
        window.initializeRenderer(AppState.worldWidth, AppState.worldHeight);
    }

    // Update UI
    if (window.updateUI) {
        window.updateUI(AppState);
    }

    // Render the world
    if (window.renderWorld) {
        window.renderWorld(AppState);
    }
}

// Update connection status indicator
function updateConnectionStatus(status) {
    const indicator = document.getElementById('status-indicator');
    const text = document.getElementById('connection-text');

    indicator.className = 'status-indicator ' + status;

    switch (status) {
        case 'connected':
            text.textContent = 'Connected';
            break;
        case 'connecting':
            text.textContent = 'Connecting...';
            break;
        case 'disconnected':
            text.textContent = 'Disconnected';
            break;
    }
}

// Creature selection
function selectCreature(creature) {
    AppState.selectedCreature = creature;
    if (window.showCreatureInspector) {
        window.showCreatureInspector(creature);
    }
}

function deselectCreature() {
    AppState.selectedCreature = null;
    if (window.hideCreatureInspector) {
        window.hideCreatureInspector();
    }
}

// Initialize the application
document.addEventListener('DOMContentLoaded', () => {
    console.log('Evolution Simulator starting...');

    // Connect to WebSocket
    connectWebSocket();

    // Set up creature inspector close button
    const closeButton = document.getElementById('close-inspector');
    if (closeButton) {
        closeButton.addEventListener('click', deselectCreature);
    }
});

// Export functions for use in other modules
window.AppState = AppState;
window.selectCreature = selectCreature;
window.deselectCreature = deselectCreature;
window.sendMessage = sendMessage;
