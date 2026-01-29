-- NET Component Names
-- Centralized component name definitions for consistency across modules

local Components = {
    -- Core sync marker component (on entities that replicate)
    MARKER = "NetworkSync",
    
    -- Outbound message entity (created by outbound system, consumed by server/client)
    OUTBOUND = "NetSyncOutbound",
    
    -- Inbound message entity (created by server/client, consumed by inbound system)
    INBOUND = "NetSyncInbound",
    
    -- Client-side prediction state component
    PREDICTION = "PredictionState",
    
    -- Interpolation target for remote entities
    INTERPOLATION = "InterpolationTarget",
}

return Components
