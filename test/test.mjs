import { describe, it } from 'node:test';
import assert from 'node:assert';
import { readFile } from 'node:fs/promises';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const { heifToJpeg } = await import('../index.js');

describe('heifToJpeg', () => {
  it('converts HEIC to JPEG', async () => {
    const heic = await readFile(join(__dirname, 'fixtures', 'sample.heic'));
    const jpeg = await heifToJpeg(heic);
    assert.ok(Buffer.isBuffer(jpeg));
    assert.ok(jpeg.length > 0);
    // JPEG starts with SOI marker
    assert.strictEqual(jpeg[0], 0xff);
    assert.strictEqual(jpeg[1], 0xd8);
  });

  it('respects quality option', async () => {
    const heic = await readFile(join(__dirname, 'fixtures', 'sample.heic'));
    const low = await heifToJpeg(heic, { quality: 10 });
    const high = await heifToJpeg(heic, { quality: 95 });
    // Lower quality should produce smaller output
    assert.ok(low.length < high.length);
  });

  it('rejects invalid input', async () => {
    await assert.rejects(() => heifToJpeg(Buffer.from('not a heif file')));
  });

  it('uses default quality when no options given', async () => {
    const heic = await readFile(join(__dirname, 'fixtures', 'sample.heic'));
    const jpeg = await heifToJpeg(heic);
    assert.ok(jpeg.length > 0);
  });
});
