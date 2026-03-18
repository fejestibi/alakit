// Csak a vízjelezés és Canvas rajzolás felelőssége van JS-ben
const canvas = document.getElementById('wm_canvas');
const ctx = canvas.getContext('2d');
let isDrawing = false;

// Alapállapot
ctx.lineWidth = 4;
ctx.lineCap = 'round';
ctx.strokeStyle = '#ffffff';

// --- Rajzolás Események ---
canvas.addEventListener('mousedown', (e) => {
    isDrawing = true;
    ctx.beginPath();
    ctx.moveTo(e.offsetX, e.offsetY);
});

canvas.addEventListener('mousemove', (e) => {
    if (isDrawing) {
        ctx.lineTo(e.offsetX, e.offsetY);
        ctx.stroke();
    }
});

canvas.addEventListener('mouseup', () => isDrawing = false);
canvas.addEventListener('mouseleave', () => isDrawing = false);

// Törlés gomb
document.getElementById('btn_clear_watermark').addEventListener('click', () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
});

// Küldés gomb (A LÉNYEG)
document.getElementById('btn_apply_watermark').addEventListener('click', () => {
    // 1. Kiszedjük a RAW RGBA pixeleket a vászonról
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    const pixels = new Uint8Array(imageData.data.buffer);
    
    // Fejléc
    const headerStr = JSON.stringify({ w: canvas.width, h: canvas.height });
    const encoder = new TextEncoder();
    const headerBytes = encoder.encode(headerStr);
    
    // Csomag összeállítása [Hossz 4 bájt] + [Header] + [Pixelek]
    const payload = new Uint8Array(4 + headerBytes.length + pixels.length);
    payload[0] = headerBytes.length & 0xFF;
    payload[1] = (headerBytes.length >> 8) & 0xFF;
    payload[2] = (headerBytes.length >> 16) & 0xFF;
    payload[3] = (headerBytes.length >> 24) & 0xFF;
    
    payload.set(headerBytes, 4);
    payload.set(pixels, 4 + headerBytes.length);

    console.log(`Sending Watermark via Binary IPC... (${pixels.length} bytes)`);
    
    // ULTRA-FAST BINARY IPC HÍVÁS
    window.alakit_binary(`image/watermark`, payload);
});
