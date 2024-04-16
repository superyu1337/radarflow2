// Colors
const localColor = "#109856"
const teamColor = "#68a3e5"
const enemyColor = "#ec040b"
const bombColor = "#eda338"
const textColor = "#d1d1d1"

// Settings
shouldZoom = true
drawStats = false

// Common
canvas = null
ctx = null

// radarflow specific
radarData = null
freq = 0
image = null
map = null
mapName = null
loaded = false
entityData = null
update = false

/// Radarflow zoom in
zoomSet = false
safetyBound = 50
boundingRect = null

// networking
websocket = null
if (location.protocol == 'https:') {
    websocketAddr = `wss://${window.location.host}/ws`
} else {
    websocketAddr = `ws://${window.location.host}/ws`
}
//websocketAddr = "ws://192.168.0.235:8000/ws"

// Util functions
const clamp = (num, min, max) => Math.min(Math.max(num, min), max);
const degreesToRadians = (degrees) => degrees * (Math.PI/180);
function makeBoundingRect(x1, y1, x2, y2, aspectRatio) {
    const topLeftX = x1;
    const topLeftY = y1;
    const bottomRightX = x2;
    const bottomRightY = y2;

    const width = bottomRightX - topLeftX;
    const height = bottomRightY - topLeftY;

    let newWidth, newHeight;
    if (width / height > aspectRatio) {
        // Wider rectangle
        newHeight = width / aspectRatio;
        newWidth = width;
    } else {
        // Taller rectangle
        newWidth = height * aspectRatio;
        newHeight = height;
    }

    const centerX = (topLeftX + bottomRightX) / 2;
    const centerY = (topLeftY + bottomRightY) / 2;

    const rectMinX = centerX - newWidth / 2;
    const rectMaxX = centerX + newWidth / 2;
    const rectMinY = centerY - newHeight / 2;
    const rectMaxY = centerY + newHeight / 2;

    const boundingRectangle = {
        x: rectMinX,
        y: rectMinY,
        width: rectMaxX - rectMinX,
        height: rectMaxY - rectMinY,
    }

    const boundingRectangle2 = {
        x: 0,
        y: 0,
        width: image.width / 1.2,
        height: image.width / 1.2,
    }

    return boundingRectangle;
}

function boundingCoordinates(coordinates, boundingRect) {
    const xScale = boundingRect.width / image.width;
    const yScale = boundingRect.height / image.height;

    const newX = (coordinates.x - boundingRect.x) / xScale;
    const newY = (coordinates.y - boundingRect.y) / yScale;

    return {x: newX, y: newY};
}

function boundingScale(value, boundingRect) {
    const scale = image.width / boundingRect.width;
    return value * scale
}

function mapCoordinates(coordinates) {
    let offset_x = coordinates.x - map.pos_x;
    let offset_y = coordinates.y - map.pos_y;

    offset_x /= map.scale;
    offset_y /= -map.scale;

    return {x: offset_x, y: offset_y}
}

function render() {
    if (update) {
        fillCanvas()
        if (loaded) {
            update = false

            // Iterate through the array and update the min/max values
            if (entityData != null && map != null && image != null && shouldZoom) {

                let minX = Infinity
                let minY = Infinity
                let maxX = -Infinity
                let maxY = -Infinity

                entityData.forEach((data) => {
                    let mapCords = null

                    if (data.Bomb !== undefined) {
                        mapCords = mapCoordinates(data.Bomb.pos)
                    } else {
                        mapCords = mapCoordinates(data.Player.pos)
                    }
    
                    minX = Math.min(minX, mapCords.x);
                    minY = Math.min(minY, mapCords.y);
                    maxX = Math.max(maxX, mapCords.x);
                    maxY = Math.max(maxY, mapCords.y);
                });


                boundingRect = makeBoundingRect(minX-safetyBound, minY-safetyBound, maxX+safetyBound, maxY+safetyBound, image.width/image.height)

                zoomSet = true
            } else if (zoomSet) {
                zoomSet = false
            }

            drawImage()

            if (entityData != null) {
                entityData.forEach((data) => {
                    if (data.Bomb !== undefined) {
                        drawBomb(data.Bomb.pos, data.Bomb.isPlanted)
                    } else {
                        let fillStyle = localColor

                        switch (data.Player.playerType) {
                            case "Team":
                                fillStyle = teamColor
                                break;

                            case "Enemy":
                                fillStyle = enemyColor
                                break;
                        }

                        drawEntity(
                            data.Player.pos,
                            fillStyle, 
                            data.Player.isDormant,
                            data.Player.hasBomb,
                            data.Player.yaw,
                            data.Player.hasAwp,
                            data.Player.playerType,
                            data.Player.isScoped
                        )
                    }
                });
            }

            if (radarData != null) {
                if (radarData.bombPlanted && !radarData.bombExploded && radarData.bombDefuseTimeleft >= 0) {

                    let maxWidth = 1024-128-128;
                    let timeleft = radarData.bombDefuseTimeleft;
    
                    // Base bar
                    ctx.fillStyle = "black"
                    ctx.fillRect(128, 16, maxWidth, 16)
                    
                    // Bomb timer
                    if (radarData.bombBeingDefused) {
                        if (radarData.bombCanDefuse) {
                            ctx.fillStyle = teamColor
                        } else {
                            ctx.fillStyle = enemyColor
                        }
                    } else {
                        ctx.fillStyle = bombColor
                    }

                    ctx.fillRect(130, 18, (maxWidth-2) * (timeleft / 40), 12)

                    ctx.font = "24px Arial";
                    ctx.textAlign = "center"
                    ctx.textBaseline = "middle"
                    ctx.fillStyle = textColor
                    ctx.fillText(`${timeleft.toFixed(1)}s`, 1024/2, 28+24); 
                    
                    // Defuse time lines
                    ctx.strokeStyle = "black"
                    ctx.lineWidth = 2

                    // Kit defuse
                    ctx.beginPath()
                    ctx.moveTo(128 + (maxWidth * (5 / 40)), 16)
                    ctx.lineTo(128 + (maxWidth * (5 / 40)), 32)
                    ctx.stroke()

                    // Normal defuse
                    ctx.beginPath()
                    ctx.moveTo(130 + (maxWidth-2) * (10 / 40), 16)
                    ctx.lineTo(130 + (maxWidth-2) * (10 / 40), 32)
                    ctx.stroke()

                    // Defuse stamp line
                    if (radarData.bombCanDefuse) {
                        console.log(radarData.bombDefuseEnd)
                        ctx.strokeStyle = "green"
                        ctx.beginPath()
                        ctx.moveTo(130 + (maxWidth-2) * (radarData.bombDefuseEnd / 40), 16)
                        ctx.lineTo(130 + (maxWidth-2) * (radarData.bombDefuseEnd / 40), 32)
                        ctx.stroke()
                    }
                }
            }

        } else {
            if (websocket != null) {
                ctx.font = "100px Arial";
                ctx.textAlign = "center"
                ctx.textBaseline = "middle"
                ctx.fillStyle = textColor
                ctx.fillText("Not on a server", 1024/2, 1024/2); 
            } else {
                ctx.font = "100px Arial";
                ctx.textAlign = "center"
                ctx.fillStyle = textColor
                ctx.fillText("Disconnected", 1024/2, 1024/2); 
            }
        }

        if (drawStats) {
            ctx.font = "16px Arial";
            ctx.textAlign = "left"
            ctx.fillStyle = textColor
            ctx.lineWidth = 2;
            ctx.strokeStyle = "black"
            ctx.strokeText(`${freq} Hz`, 2, 18)
            ctx.fillText(`${freq} Hz`, 2, 18)
        }
    }

    if (websocket != null) {
        websocket.send("requestInfo");
    }
}

function fillCanvas() {
    ctx.fillStyle = "#0f0f0f";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
}

function drawImage() {
    if (image == null || canvas == null)
        return

    if (zoomSet != false && boundingRect.x != null) {
        ctx.drawImage(image, boundingRect.x, boundingRect.y, boundingRect.width, boundingRect.height, 0, 0, canvas.width, canvas.height)
    } else {
        ctx.drawImage(image, 0, 0, image.width, image.height, 0, 0, canvas.width, canvas.height)
    }
}

function drawBomb(pos, planted) {
    if (map == null)
        return

    if (zoomSet) {
        pos = boundingCoordinates(mapCoordinates(pos), boundingRect)
        size = boundingScale(5, boundingRect);
    } else {
        pos = mapCoordinates(pos)
        size = 5
    }

    ctx.beginPath();
    ctx.arc(pos.x, pos.y, size, 0, 2 * Math.PI);
    ctx.fillStyle = bombColor;
    ctx.fill();
    ctx.closePath();

    if (planted && ((new Date().getTime() / 1000) % 1) > 0.5) {
        ctx.strokeStyle = enemyColor
        ctx.lineWidth = 1;
        ctx.stroke()
    }
}

function drawEntity(pos, fillStyle, dormant, hasBomb, yaw, hasAwp, playerType, isScoped) {
    if (map == null)
        return

    if (zoomSet) {
        pos = boundingCoordinates(mapCoordinates(pos), boundingRect)
        circleRadius = boundingScale(7, boundingRect);
        distance = circleRadius + boundingScale(2, boundingRect);
        radius = distance + boundingScale(2, boundingRect)
        arrowWidth = 35
    } else {
        pos = mapCoordinates(pos)
        circleRadius = 7
        distance = circleRadius + 2
        radius = distance + 5;
        arrowWidth = 35;
    }

    if (dormant) {
        ctx.font = "20px Arial";
        ctx.textAlign = "center"
        ctx.fillStyle = fillStyle
        ctx.fillText("?", pos.x, pos.y); 
    } else {

        if (hasAwp) {
            ctx.beginPath();
            ctx.arc(pos.x, pos.y, circleRadius, 0, 2 * Math.PI);
            ctx.fillStyle = "orange";
            ctx.fill();
            circleRadius -= 2;
        }

        // Draw circle
        ctx.beginPath();
        ctx.arc(pos.x, pos.y, circleRadius, 0, 2 * Math.PI);
        ctx.fillStyle = fillStyle;
        ctx.fill();

        if (hasAwp && false) {

            let style = "yellow"

            if (playerType == "Enemy") {
                style = "orange"
            }

            ctx.beginPath();
            ctx.arc(pos.x, pos.y, circleRadius / 1.5, 0, 2 * Math.PI);
            ctx.fillStyle = style;
            ctx.fill();
        }

        if (hasBomb) {
            ctx.beginPath();
            ctx.arc(pos.x, pos.y, circleRadius / 2, 0, 2 * Math.PI);
            ctx.fillStyle = bombColor;
            ctx.fill();
        }


        ctx.closePath();

        // Calculate arrowhead points


        const arrowHeadX = pos.x + radius * Math.cos(yaw * (Math.PI / 180))
        const arrowHeadY = pos.y - radius * Math.sin(yaw * (Math.PI / 180))

        const arrowCornerX1 = pos.x + distance * Math.cos((yaw - arrowWidth) * (Math.PI / 180))
        const arrowCornerY1 = pos.y - distance * Math.sin((yaw - arrowWidth) * (Math.PI / 180))

        const arrowCornerX2 = pos.x + distance * Math.cos((yaw + arrowWidth) * (Math.PI / 180))
        const arrowCornerY2 = pos.y - distance * Math.sin((yaw + arrowWidth) * (Math.PI / 180))


        const cicleYaw = 90-yaw
        const startAngle = degreesToRadians(cicleYaw-arrowWidth)-Math.PI/2
        const endAngle = degreesToRadians(cicleYaw+arrowWidth)-Math.PI/2

        // Draw arrow

        /// Backside of the arrow
        ctx.beginPath();
        ctx.arc(pos.x, pos.y, distance, startAngle, endAngle)

        /// Draw from corners to arrowhead
        ctx.lineTo(arrowCornerX1, arrowCornerY1);
        ctx.lineTo(arrowHeadX, arrowHeadY);
        ctx.lineTo(arrowCornerX2, arrowCornerY2);
        ctx.closePath()

        ctx.fillStyle = 'white'
        ctx.fill();

        if (isScoped) {
            const lineOfSightX = arrowHeadX + 1024 * Math.cos(yaw * (Math.PI / 180))
            const lineOfSightY = arrowHeadY - 1024 * Math.sin(yaw * (Math.PI / 180))
            ctx.beginPath();
            ctx.moveTo(arrowHeadX, arrowHeadY);
            ctx.lineTo(lineOfSightX, lineOfSightY);

            if (playerType == "Enemy")
                ctx.strokeStyle = enemyColor
            else
                ctx.strokeStyle = teamColor

            ctx.strokeWidth = 1;
            ctx.stroke();
        }
    }
}

function loadMap(mapName) {
    console.log(`[radarflow] loading map ${mapName}`)
    loaded = true;
    const map_img = new Image();
    map_img.src = `assets/image/${mapName}_radar_psd.png`;

    fetch(`assets/json/${mapName}.json`)
        .then(response => response.json())
        .then(data => {
            map = data;
        })
        .catch(error => {
            console.error('Error loading JSON file:', error);
        });

    map_img.onload = () => {
        image = map_img;
        update = true;
    };
}

function unloadMap() {
    console.log("[radarflow] unloading map")
    ctx.clearRect(0, 0, canvas.width, canvas.height)
    map = null
    mapName = null
    loaded = false,
    update = true
    requestAnimationFrame(render);
}

function connect() {
    if (websocket == null) {
        let socket = new WebSocket(websocketAddr)

        socket.onopen = () => {
            console.log("[radarflow] Connection established")
            websocket.send("requestInfo");
        };
        
        socket.onmessage = (event) => {
            if (event.data == "error") {
                console.log("[radarflow] Server had an unknown error")
            } else {
                let data = JSON.parse(event.data);
                radarData = data; 
                freq = data.freq;

                if (data.ingame == false) {
                    mapName = null
                    entityData = null

                    if (loaded)
                        unloadMap()
                } else {
                    if (!loaded) {
                        mapName = data.mapName
                        entityData = data.entityData
                        loadMap(mapName)
                    } else {
                        entityData = data.entityData
                    }
                }

                update = true
                requestAnimationFrame(render);
            }
        };
        
        socket.onclose = (event) => {
            if (event.wasClean) {
                console.log("[radarflow] connection closed");
            } else {
                console.log("[radarflow] connection died");
            }

            playerData = null
            websocket = null
            unloadMap()

            setTimeout(function() {
                connect();
            }, 1000);
        };
    
        socket.onerror = (error) => {
            console.log(`[radarflow] websocket error: ${error}`);
        };

        websocket = socket;
    } else {
        setTimeout(() => {
            connect();
        }, 1000);
    }
}

addEventListener("DOMContentLoaded", (e) => {
    document.getElementById("zoomCheck").checked = true;


    canvas = document.getElementById('canvas');
    canvas.width = 1024;
    canvas.height = 1024;
    canvasAspectRatio = canvas.width / canvas.height
    ctx = canvas.getContext('2d');

    console.log(`[radarflow] connecting to ${websocketAddr}`)
    connect()
});

function toggleZoom() {
    shouldZoom = !shouldZoom
}

function toggleStats() {
    drawStats = !drawStats
}