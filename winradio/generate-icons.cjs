const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const svg = `
<svg width="256" height="256" viewBox="0 0 256 256" fill="none" xmlns="http://www.w3.org/2000/svg">
  <rect width="256" height="256" rx="32" fill="url(#gradient)"/>
  <path d="M128 64V192M64 128H192" stroke="white" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
  <defs>
    <linearGradient id="gradient" x1="0" y1="0" x2="256" y2="256" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#3B82F6"/>
      <stop offset="100%" stop-color="#8B5CF6"/>
    </linearGradient>
  </defs>
</svg>
`;

// ICO file format writer
function createIco(pngBuffers, sizes) {
  // ICO header (6 bytes)
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); // reserved
  header.writeUInt16LE(1, 2); // type: 1 = ICO
  header.writeUInt16LE(sizes.length, 4); // number of images

  // Directory entries (16 bytes each)
  const dirEntries = [];
  let imageOffset = 6 + 16 * sizes.length;
  
  for (let i = 0; i < sizes.length; i++) {
    const size = sizes[i];
    const png = pngBuffers[i];
    const entry = Buffer.alloc(16);
    entry.writeUInt8(size === 256 ? 0 : size, 0); // width (0 = 256)
    entry.writeUInt8(size === 256 ? 0 : size, 1); // height (0 = 256)
    entry.writeUInt8(0, 2); // color count
    entry.writeUInt8(0, 3); // reserved
    entry.writeUInt16LE(1, 4); // color planes
    entry.writeUInt16LE(32, 6); // bits per pixel
    entry.writeUInt32LE(png.length, 8); // size of image data
    entry.writeUInt32LE(imageOffset, 12); // offset of image data
    dirEntries.push(entry);
    imageOffset += png.length;
  }

  return Buffer.concat([header, ...dirEntries, ...pngBuffers]);
}

async function generateIcons() {
  const iconsDir = path.join(__dirname, 'src-tauri', 'icons');
  if (!fs.existsSync(iconsDir)) {
    fs.mkdirSync(iconsDir, { recursive: true });
  }

  // Generate PNG for tray icon (16x16, 24x24, 32x32, 48x48, 256x256)
  const sizes = [16, 24, 32, 48, 256];
  for (const size of sizes) {
    await sharp(Buffer.from(svg))
      .resize(size, size)
      .png()
      .toFile(path.join(iconsDir, `icon-${size}.png`));
  }

  // Generate proper ICO file with multiple sizes
  const icoSizes = [16, 24, 32, 48, 64, 128, 256];
  const pngBuffers = [];
  
  for (const size of icoSizes) {
    const buffer = await sharp(Buffer.from(svg))
      .resize(size, size)
      .png()
      .toBuffer();
    pngBuffers.push(buffer);
  }

  const icoBuffer = createIco(pngBuffers, icoSizes);
  fs.writeFileSync(path.join(iconsDir, 'icon.ico'), icoBuffer);

  // Tray icon (16x16 monochrome for Windows)
  await sharp(Buffer.from(svg))
    .resize(16, 16)
    .grayscale()
    .png()
    .toFile(path.join(iconsDir, 'tray-icon.png'));

  console.log('Icons generated successfully!');
}

generateIcons().catch(console.error);