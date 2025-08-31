import init, { sift, match_descriptors_topk, resize_image, rgba_to_gray } from './sift.js';

const fileA = document.getElementById('fileA');
const fileB = document.getElementById('fileB');
const runBtn = document.getElementById('runBtn');
const statsEl = document.getElementById('stats');
const topKInput = document.getElementById('topK');

const canvasA = document.getElementById('canvasA');
const canvasB = document.getElementById('canvasB');
const ctxA = canvasA.getContext('2d');
const ctxB = canvasB.getContext('2d');

const matchCanvas = document.getElementById('matchCanvas');
const matchCtx = matchCanvas.getContext('2d');

let imageDataA = null;
let imageDataB = null;

function setStatus(msg) { statsEl.textContent = msg; }

async function ensureWasm() {
  await init(new URL('./sift_bg.wasm', import.meta.url));
}


function grayscaleToImageData(grayscaleData, width, height) {
  const imageData = new ImageData(width, height);
  const rgba = imageData.data;
  
  for (let i = 0; i < grayscaleData.length; i++) {
    const gray = grayscaleData[i];
    rgba[i * 4] = gray;     // R
    rgba[i * 4 + 1] = gray; // G
    rgba[i * 4 + 2] = gray; // B
    rgba[i * 4 + 3] = 255;  // A
  }
  
  return imageData;
}

async function loadAndResizeImage(file, canvas, ctx, targetSize = 400) {
  const img = new Image();
  img.src = URL.createObjectURL(file);
  await img.decode();
  
  const originalWidth = img.naturalWidth;
  const originalHeight = img.naturalHeight;
  
  // Load original image to get pixel data
  const tempCanvas = document.createElement('canvas');
  const tempCtx = tempCanvas.getContext('2d');
  tempCanvas.width = originalWidth;
  tempCanvas.height = originalHeight;
  tempCtx.drawImage(img, 0, 0);
  
  const originalImageData = tempCtx.getImageData(0, 0, originalWidth, originalHeight);
  await ensureWasm(); // Ensure WASM is loaded

  const grayscaleData = rgba_to_gray(originalImageData.data, originalWidth, originalHeight);
  const resizeResult = resize_image(grayscaleData, originalWidth, originalHeight, targetSize);
  
  const resizedWidth = resizeResult.width;
  const resizedHeight = resizeResult.height;
  const resizedGrayscale = resizeResult.data;
  
  // Convert back to ImageData for display
  const resizedImageData = grayscaleToImageData(resizedGrayscale, resizedWidth, resizedHeight);
  
  // Display on canvas
  canvas.width = resizedWidth;
  canvas.height = resizedHeight;
  ctx.putImageData(resizedImageData, 0, 0);
  
  return resizedImageData;
}

async function loadURLIntoCanvas(url, canvas, ctx) {
  const img = new Image();
  img.src = url; // a.jpg or b.jpg next to index.html
  await img.decode();
  canvas.width = img.naturalWidth;
  canvas.height = img.naturalHeight;
  ctx.drawImage(img, 0, 0);
  return ctx.getImageData(0, 0, canvas.width, canvas.height);
}

function enableIfReady() {
  const ready = imageDataA && imageDataB;
  runBtn.disabled = !ready;
  if (ready) setStatus('Ready. Adjust "Top matches" and click "Run Matching", or choose different images.');
}

function getTopK() {
  const v = parseInt(topKInput.value, 10);
  if (!Number.isFinite(v) || v <= 0) return 1;
  return Math.min(Math.max(v, 1), 2000);
}

fileA.addEventListener('change', async (e) => {
  const file = e.target.files && e.target.files[0];
  if (!file) return;
  
  setStatus('Resizing Image A to 400px...');
  try {
    imageDataA = await loadAndResizeImage(file, canvasA, ctxA, 400);
    setStatus(`Loaded and resized A (${canvasA.width}x${canvasA.height}).`);
    enableIfReady();
  } catch (error) {
    console.error('Error processing Image A:', error);
    setStatus('Error processing Image A. Please try again.');
  }
});

fileB.addEventListener('change', async (e) => {
  const file = e.target.files && e.target.files[0];
  if (!file) return;
  
  setStatus('Resizing Image B to 400px...');
  try {
    imageDataB = await loadAndResizeImage(file, canvasB, ctxB, 400);
    setStatus(`Loaded and resized B (${canvasB.width}x${canvasB.height}).`);
    enableIfReady();
  } catch (error) {
    console.error('Error processing Image B:', error);
    setStatus('Error processing Image B. Please try again.');
  }
});

// a.jpg and b.jpg on page load
(async function loadDefaults() {
  try {
    setStatus('Loading defaults (a.jpg, b.jpg)…');
    imageDataA = await loadURLIntoCanvas('./a.jpg', canvasA, ctxA);
    imageDataB = await loadURLIntoCanvas('./b.jpg', canvasB, ctxB);
    enableIfReady();
  } catch (e) {
    console.error('Failed to load default images:', e);
    setStatus('Could not load defaults. Please choose images.');
  }
})();

function drawMatches(imgA, imgB, kpsA, kpsB, pairs) {
  // Side-by-side canvas
  matchCanvas.width = imgA.width + imgB.width;
  matchCanvas.height = Math.max(imgA.height, imgB.height);
  matchCtx.clearRect(0, 0, matchCanvas.width, matchCanvas.height);

  matchCtx.putImageData(imgA, 0, 0);
  matchCtx.putImageData(imgB, imgA.width, 0);

  const N = pairs.length / 2;

  for (let i = 0; i < N; i++) {
    const ia = pairs[2 * i];
    const ib = pairs[2 * i + 1];

    const ax = kpsA[ia * 6];
    const ay = kpsA[ia * 6 + 1];
    const bx = kpsB[ib * 6] + imgA.width; // shift to right image
    const by = kpsB[ib * 6 + 1];

    const hue = (i * 137) % 360;
    matchCtx.strokeStyle = `hsl(${hue}, 90%, 55%)`;
    matchCtx.lineWidth = 1.2;

    matchCtx.beginPath();
    matchCtx.moveTo(ax, ay);
    matchCtx.lineTo(bx, by);
    matchCtx.stroke();

    matchCtx.fillStyle = matchCtx.strokeStyle;
    matchCtx.beginPath(); matchCtx.arc(ax, ay, 2, 0, Math.PI * 2); matchCtx.fill();
    matchCtx.beginPath(); matchCtx.arc(bx, by, 2, 0, Math.PI * 2); matchCtx.fill();
  }
}

runBtn.addEventListener('click', async () => {
  if (!imageDataA || !imageDataB) return;

  runBtn.disabled = true;
  setStatus('Initializing WASM...');
  await ensureWasm();

  setStatus('Converting to grayscale...');
  const grayA = rgba_to_gray(imageDataA.data, imageDataA.width, imageDataA.height);
  const grayB = rgba_to_gray(imageDataB.data, imageDataB.width, imageDataB.height);

  setStatus('Running SIFT...');
  const scales = 3;
  const resA = sift(grayA, imageDataA.width, imageDataA.height, scales);
  const resB = sift(grayB, imageDataB.width, imageDataB.height, scales);

  const kpsA = resA.keypoints;      // flat [x,y,oct,level,sigma,angle,...]
  const kpsB = resB.keypoints;
  const descA = resA.descriptors;   // flat 128 per keypoint
  const descB = resB.descriptors;

  const nA = (kpsA.length / 6) | 0;
  const nB = (kpsB.length / 6) | 0;

  const topK = getTopK();
  setStatus(`A: ${nA} kps | B: ${nB} kps. Matching top-${topK}...`);

  const ratio = 0.75;
  const crossCheck = true;
  const pairs = match_descriptors_topk(descA, descB, 128, ratio, crossCheck, topK);

  if (pairs.length === 0) {
    matchCanvas.width = imageDataA.width + imageDataB.width;
    matchCanvas.height = Math.max(imageDataA.height, imageDataB.height);
    matchCtx.clearRect(0, 0, matchCanvas.width, matchCanvas.height);
    matchCtx.putImageData(imageDataA, 0, 0);
    matchCtx.putImageData(imageDataB, imageDataA.width, 0);
    setStatus('No matches passed the filters.');
    runBtn.disabled = false;
    return;
  }

  drawMatches(imageDataA, imageDataB, kpsA, kpsB, pairs);
  setStatus(`Matches shown: ${(pairs.length / 2) | 0} (Top-${topK})`);

  runBtn.disabled = false;
});