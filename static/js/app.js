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
    food: [],
    metrics: {},

    // UI state
    selectedCreature: null,
    subscribedCreatureId: null,

    // Playback state
    playbackMode: 'live', // 'live' or 'paused'
    stateBuffer: [],
    currentBufferIndex: -1,
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
        case 'creature_details':
            handleCreatureDetails(message);
            break;
        case 'creature_update':
            handleCreatureUpdate(message);
            break;
        default:
            console.warn('Unknown message type:', message.type);
    }
}

// Handle creature details message
function handleCreatureDetails(data) {
    if (window.updateCreatureDetails) {
        window.updateCreatureDetails(data);
    }
}

// Handle creature update message (real-time updates)
function handleCreatureUpdate(message) {
    if (window.updateCreatureDetails) {
        window.updateCreatureDetails(message.details);
    }
}

// Handle update message
function handleUpdate(message) {
    const stateSnapshot = {
        metrics: message.metrics || {},
        creatures: message.creatures || [],
        food: message.food || [],
    };

    if (AppState.playbackMode === 'paused') {
        // Add to buffer when paused
        AppState.stateBuffer.push(stateSnapshot);
    } else {
        // Live mode: update immediately
        AppState.metrics = stateSnapshot.metrics;
        AppState.creatures = stateSnapshot.creatures;
        AppState.food = stateSnapshot.food;

        // Update UI
        if (window.updateUI) {
            window.updateUI(AppState);
        }

        // Render the world
        if (window.renderWorld) {
            window.renderWorld(AppState);
        }
    }
}

// Handle full state message
function handleFullState(message) {
    const stateSnapshot = {
        metrics: message.metrics || {},
        creatures: message.creatures || [],
        food: message.food || [],
    };

    // Extract world dimensions (sent directly in message, not nested in world object)
    AppState.worldWidth = message.world_width || 0;
    AppState.worldHeight = message.world_height || 0;

    console.log(`World size: ${AppState.worldWidth}x${AppState.worldHeight}`);
    console.log(`Creatures: ${AppState.creatures.length}`);

    // Initialize renderer with world dimensions
    if (window.initializeRenderer) {
        window.initializeRenderer(AppState.worldWidth, AppState.worldHeight);
    }

    if (AppState.playbackMode === 'paused') {
        // Add to buffer when paused
        AppState.stateBuffer.push(stateSnapshot);
    } else {
        // Live mode: update immediately
        AppState.metrics = stateSnapshot.metrics;
        AppState.creatures = stateSnapshot.creatures;
        AppState.food = stateSnapshot.food;

        // Update UI
        if (window.updateUI) {
            window.updateUI(AppState);
        }

        // Render the world
        if (window.renderWorld) {
            window.renderWorld(AppState);
        }
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

// Playback controls
function pausePlayback() {
    if (AppState.playbackMode === 'paused') return;

    AppState.playbackMode = 'paused';
    // Initialize buffer with current state
    AppState.stateBuffer = [{
        metrics: AppState.metrics,
        creatures: AppState.creatures,
        food: AppState.food,
    }];
    AppState.currentBufferIndex = 0;

    console.log('Playback paused');
    updatePlaybackControls();
}

function stepBackward() {
    if (AppState.playbackMode !== 'paused' || AppState.currentBufferIndex <= 0) return;

    AppState.currentBufferIndex--;
    renderBufferedState(AppState.currentBufferIndex);
    updatePlaybackControls();
}

function stepForward() {
    if (AppState.playbackMode !== 'paused') return;
    if (AppState.currentBufferIndex >= AppState.stateBuffer.length - 1) return;

    AppState.currentBufferIndex++;
    renderBufferedState(AppState.currentBufferIndex);
    updatePlaybackControls();
}

function goLive() {
    if (AppState.playbackMode === 'live') return;

    AppState.playbackMode = 'live';
    AppState.stateBuffer = [];
    AppState.currentBufferIndex = -1;

    console.log('Resumed live playback');
    updatePlaybackControls();
}

function renderBufferedState(index) {
    if (index < 0 || index >= AppState.stateBuffer.length) return;

    const state = AppState.stateBuffer[index];
    AppState.metrics = state.metrics;
    AppState.creatures = state.creatures;
    AppState.food = state.food;

    // Update UI
    if (window.updateUI) {
        window.updateUI(AppState);
    }

    // Render the world
    if (window.renderWorld) {
        window.renderWorld(AppState);
    }
}

function updatePlaybackControls() {
    const pauseBtn = document.getElementById('pause-btn');
    const stepBackBtn = document.getElementById('step-back-btn');
    const stepForwardBtn = document.getElementById('step-forward-btn');
    const goLiveBtn = document.getElementById('go-live-btn');

    if (AppState.playbackMode === 'paused') {
        // Show/hide appropriate buttons
        if (pauseBtn) pauseBtn.style.display = 'none';
        if (goLiveBtn) goLiveBtn.style.display = 'block';

        // Enable/disable step buttons based on position
        if (stepBackBtn) {
            stepBackBtn.disabled = AppState.currentBufferIndex <= 0;
        }
        if (stepForwardBtn) {
            stepForwardBtn.disabled = AppState.currentBufferIndex >= AppState.stateBuffer.length - 1;
        }
    } else {
        // Live mode
        if (pauseBtn) pauseBtn.style.display = 'block';
        if (goLiveBtn) goLiveBtn.style.display = 'none';
        if (stepBackBtn) stepBackBtn.disabled = true;
        if (stepForwardBtn) stepForwardBtn.disabled = true;
    }
}

// Creature selection
function selectCreature(creature) {
    AppState.selectedCreature = creature;
    AppState.subscribedCreatureId = creature.id;

    // Request detailed creature data
    sendMessage({
        type: 'get_creature_details',
        creature_id: creature.id
    });

    // Subscribe to real-time updates for this creature
    sendMessage({
        type: 'subscribe_creature',
        creature_id: creature.id
    });

    if (window.showCreatureInspector) {
        window.showCreatureInspector(creature);
    }
}

function deselectCreature() {
    AppState.selectedCreature = null;
    AppState.subscribedCreatureId = null;

    // Unsubscribe from creature updates
    sendMessage({
        type: 'subscribe_creature',
        creature_id: null
    });

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

    // Set up playback control buttons
    const pauseBtn = document.getElementById('pause-btn');
    const stepBackBtn = document.getElementById('step-back-btn');
    const stepForwardBtn = document.getElementById('step-forward-btn');
    const goLiveBtn = document.getElementById('go-live-btn');

    if (pauseBtn) {
        pauseBtn.addEventListener('click', pausePlayback);
    }
    if (stepBackBtn) {
        stepBackBtn.addEventListener('click', stepBackward);
    }
    if (stepForwardBtn) {
        stepForwardBtn.addEventListener('click', stepForward);
    }
    if (goLiveBtn) {
        goLiveBtn.addEventListener('click', goLive);
    }
});

// Export functions for use in other modules
window.AppState = AppState;
window.selectCreature = selectCreature;
window.deselectCreature = deselectCreature;
window.sendMessage = sendMessage;
window.pausePlayback = pausePlayback;
window.stepBackward = stepBackward;
window.stepForward = stepForward;
window.goLive = goLive;
window.updatePlaybackControls = updatePlaybackControls;
