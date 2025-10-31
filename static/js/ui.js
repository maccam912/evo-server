// Update the statistics panel
function updateUI(appState) {
    updateStatistics(appState.metrics);
}

// Update statistics panel
function updateStatistics(metrics) {
    // Population and generation
    setStatValue('stat-population', metrics.population || 0);
    setStatValue('stat-generation', metrics.generation || 0);
    setStatValue('stat-tick', metrics.tick || 0);

    // Energy
    setStatValue('stat-avg-energy', formatNumber(metrics.avg_energy || 0, 2));
    setStatValue('stat-total-energy', formatNumber(metrics.total_energy || 0, 0));

    // Lifecycle
    setStatValue('stat-births', metrics.total_births || 0);
    setStatValue('stat-deaths', metrics.total_deaths || 0);
    setStatValue('stat-avg-age', formatNumber(metrics.avg_age || 0, 1));

    // World
    const worldSize = window.AppState ?
        `${window.AppState.worldWidth} × ${window.AppState.worldHeight}` : '-';
    setStatValue('stat-world-size', worldSize);
    setStatValue('stat-total-food', formatNumber(metrics.total_food || 0, 0));
}

// Set stat value
function setStatValue(elementId, value) {
    const element = document.getElementById(elementId);
    if (element) {
        element.textContent = value;
    }
}

// Format number with specific decimal places
function formatNumber(num, decimals) {
    if (typeof num !== 'number' || isNaN(num)) return '-';
    return num.toFixed(decimals);
}

// Show creature inspector
function showCreatureInspector(creature) {
    const inspector = document.getElementById('creature-inspector');
    if (!inspector) return;

    // Populate inspector with creature data
    populateCreatureInspector(creature);

    // Show inspector
    inspector.classList.add('visible');
}

// Hide creature inspector
function hideCreatureInspector() {
    const inspector = document.getElementById('creature-inspector');
    if (inspector) {
        inspector.classList.remove('visible');
    }
}

// Populate creature inspector with data
function populateCreatureInspector(creature) {
    // Basic info
    setInfoValue('creature-id', creature.id || '-');
    setInfoValue('creature-position', `(${formatNumber(creature.x, 1)}, ${formatNumber(creature.y, 1)})`);
    setInfoValue('creature-energy', formatNumber(creature.energy, 2));
    setInfoValue('creature-age', creature.age || 0);
    setInfoValue('creature-generation', creature.generation || 0);

    // Genome
    if (creature.genome) {
        const genomeLength = creature.genome.dna ? creature.genome.dna.length : 0;
        setInfoValue('creature-genome-length', genomeLength);

        // Display genome DNA (truncated if too long)
        const genomeDisplay = document.getElementById('creature-genome');
        if (genomeDisplay && creature.genome.dna) {
            const dna = creature.genome.dna;
            if (dna.length > 500) {
                genomeDisplay.textContent = formatGenome(dna.slice(0, 500)) + '... (truncated)';
            } else {
                genomeDisplay.textContent = formatGenome(dna);
            }
        }
    } else {
        setInfoValue('creature-genome-length', '-');
        const genomeDisplay = document.getElementById('creature-genome');
        if (genomeDisplay) {
            genomeDisplay.textContent = 'No genome data';
        }
    }

    // Neural Network
    if (creature.neural_network) {
        const nn = creature.neural_network;
        setInfoValue('creature-nn-inputs', nn.input_size || '-');
        setInfoValue('creature-nn-outputs', nn.output_size || '-');

        // Hidden layers info
        if (nn.hidden_layers && nn.hidden_layers.length > 0) {
            const layerInfo = nn.hidden_layers.map(l => l.size || 0).join(', ');
            setInfoValue('creature-nn-hidden', `${nn.hidden_layers.length} (${layerInfo})`);
        } else {
            setInfoValue('creature-nn-hidden', '0');
        }
    } else {
        setInfoValue('creature-nn-inputs', '-');
        setInfoValue('creature-nn-outputs', '-');
        setInfoValue('creature-nn-hidden', '-');
    }
}

// Set info value
function setInfoValue(elementId, value) {
    const element = document.getElementById(elementId);
    if (element) {
        element.textContent = value;
    }
}

// Format genome for display (add spaces for readability)
function formatGenome(dna) {
    if (!Array.isArray(dna)) return 'Invalid genome';

    return dna.map(value => {
        if (typeof value === 'number') {
            return value.toFixed(2);
        }
        return String(value);
    }).join(' ');
}

// Toggle stats panel
function toggleStatsPanel() {
    const panel = document.getElementById('stats-panel');
    if (panel) {
        panel.classList.toggle('collapsed');
    }
}

// Restart server
function restartServer() {
    if (confirm('Are you sure you want to restart the server? This will disconnect all clients.')) {
        // Send restart command to server
        fetch('/api/restart', {
            method: 'POST'
        }).then(response => {
            if (response.ok) {
                alert('Server restart initiated. The page will reload in a few seconds.');
                // Reload the page after a delay to allow the server to restart
                setTimeout(() => {
                    window.location.reload();
                }, 3000);
            } else {
                alert('Failed to restart server.');
            }
        }).catch(error => {
            console.error('Error restarting server:', error);
            alert('Error restarting server.');
        });
    }
}

// Initialize UI event listeners
function initializeUI() {
    // Set up toggle panel button
    const toggleButton = document.getElementById('toggle-panel');
    if (toggleButton) {
        toggleButton.addEventListener('click', toggleStatsPanel);
    }

    // Set up restart server button
    const restartButton = document.getElementById('restart-server');
    if (restartButton) {
        restartButton.addEventListener('click', restartServer);
    }
}

// Initialize UI when DOM is loaded
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeUI);
} else {
    initializeUI();
}

// Export functions
window.updateUI = updateUI;
window.showCreatureInspector = showCreatureInspector;
window.hideCreatureInspector = hideCreatureInspector;
window.toggleStatsPanel = toggleStatsPanel;
