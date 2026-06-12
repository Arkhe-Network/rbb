import { ethers } from "ethers";

// Constants matching CathedralSPHINCSVerifier.sol (C13)
const N = 16;
const W = 8;
const L = 43;
const K = 8;
const A = 16;
const H_TOTAL = 24;
const H_PER_LAYER = 12;

function keccak256(data: Uint8Array): Uint8Array {
  return ethers.getBytes(ethers.keccak256(data));
}

function concat(buffers: Uint8Array[]): Uint8Array {
  const totalLength = buffers.reduce((acc, b) => acc + b.length, 0);
  const result = new Uint8Array(totalLength);
  let offset = 0;
  for (const b of buffers) {
    result.set(b, offset);
    offset += b.length;
  }
  return result;
}

function leftAlign16(data: Uint8Array): Uint8Array {
    if (data.length !== 16) throw new Error("Expected 16 bytes");
    const result = new Uint8Array(32);
    result.set(data, 0);
    return result;
}

export function baseW(message: Uint8Array, outLen: number): number[] {
    let inIdx = 0;
    let total = 0;
    let bits = 0;
    const digits: number[] = [];

    for (let i = 0; i < outLen; i++) {
        if (bits < 3) {
            total = (total << 8) | message[inIdx];
            inIdx++;
            bits += 8;
        }
        bits -= 3;
        digits.push((total >> bits) & 7);
    }
    return digits;
}

export function generateMockSignature(messageStr: string) {
  const messageBytes = ethers.getBytes(ethers.id(messageStr)); // 32 bytes message hash

  const randomizer = new Uint8Array(16);
  for(let i = 0; i < 16; i++) randomizer[i] = i;

  const randomizer32 = leftAlign16(randomizer);
  const mdHashStr = ethers.keccak256(concat([randomizer32, messageBytes]));
  const mdBytes = ethers.getBytes(mdHashStr);

  const mdBigInt = BigInt(mdHashStr);
  const idx_tree = Number((mdBigInt >> 128n) % BigInt(1 << (H_TOTAL - H_PER_LAYER)));
  const idx_leaf = Number(mdBigInt % BigInt(1 << H_PER_LAYER));

  const sigParts: Uint8Array[] = [];
  sigParts.push(randomizer);

  const forsRoots: Uint8Array[] = [];
  for (let i = 0; i < K; i++) {
    const leaf = new Uint8Array(16);
    for(let l=0; l<16; l++) leaf[l] = 0x11 + i;
    sigParts.push(leaf);

    const authPath: Uint8Array[] = [];
    for (let j = 0; j < A; j++) {
      const node = new Uint8Array(16);
      for(let l=0; l<16; l++) node[l] = 0xAA + j;
      authPath.push(node);
      sigParts.push(node);
    }

    const iBytes = new Uint8Array(32);
    const iDataView = new DataView(iBytes.buffer);
    iDataView.setUint32(28, i);

    const leafIdxHashStr = ethers.keccak256(concat([mdBytes, iBytes]));
    const leafIdx = Number(BigInt(leafIdxHashStr) % BigInt(1 << A));

    let root = leftAlign16(leaf);
    for (let j = 0; j < A; j++) {
        const sibling = leftAlign16(authPath[j]);
        if (((leafIdx >> j) & 1) === 0) {
            root = keccak256(concat([root, sibling]));
        } else {
            root = keccak256(concat([sibling, root]));
        }
    }
    forsRoots.push(root);
  }

  const forsPKStr = ethers.keccak256(concat(forsRoots));
  const forsPKBytes = ethers.getBytes(forsPKStr);

  const layer0NodeParts: Uint8Array[] = [];
  const wotsMsg0 = forsPKBytes; // 32 bytes
  const digits0 = baseW(wotsMsg0, L); // consumes 16 bytes!

  for (let i = 0; i < L; i++) {
      const chainVal = new Uint8Array(16);
      for(let l=0; l<16; l++) chainVal[l] = 0xBB + i;
      sigParts.push(chainVal);

      let currentVal = leftAlign16(chainVal);
      const digit = digits0[i];

      for (let j = digit; j < W - 1; j++) {
          currentVal = keccak256(currentVal);
      }
      layer0NodeParts.push(currentVal);
  }
  const layer0NodeStr = ethers.keccak256(concat(layer0NodeParts));
  let layer0Node = ethers.getBytes(layer0NodeStr);

  const merklePath0: Uint8Array[] = [];
  for (let i = 0; i < H_PER_LAYER; i++) {
      const node = new Uint8Array(16);
      for(let l=0; l<16; l++) node[l] = 0xCC + i;
      merklePath0.push(node);
      sigParts.push(node);
  }

  let currentLayer0Root = layer0Node;
  for (let i = 0; i < H_PER_LAYER; i++) {
      const sibling = leftAlign16(merklePath0[i]);
      if (((idx_leaf >> i) & 1) === 0) {
          currentLayer0Root = keccak256(concat([currentLayer0Root, sibling]));
      } else {
          currentLayer0Root = keccak256(concat([sibling, currentLayer0Root]));
      }
  }

  const layer1NodeParts: Uint8Array[] = [];
  const wotsMsg1 = currentLayer0Root; // 32 bytes
  const digits1 = baseW(wotsMsg1, L); // consumes 16 bytes!

  for (let i = 0; i < L; i++) {
      const chainVal = new Uint8Array(16);
      for(let l=0; l<16; l++) chainVal[l] = 0xDD + i;
      sigParts.push(chainVal);

      let currentVal = leftAlign16(chainVal);
      const digit = digits1[i];

      for (let j = digit; j < W - 1; j++) {
          currentVal = keccak256(currentVal);
      }
      layer1NodeParts.push(currentVal);
  }
  const layer1NodeStr = ethers.keccak256(concat(layer1NodeParts));
  let layer1Node = ethers.getBytes(layer1NodeStr);

  const merklePath1: Uint8Array[] = [];
  for (let i = 0; i < H_PER_LAYER; i++) {
      const node = new Uint8Array(16);
      for(let l=0; l<16; l++) node[l] = 0xEE + i;
      merklePath1.push(node);
      sigParts.push(node);
  }

  let finalRoot = layer1Node;
  for (let i = 0; i < H_PER_LAYER; i++) {
      const sibling = leftAlign16(merklePath1[i]);
      if (((idx_tree >> i) & 1) === 0) {
          finalRoot = keccak256(concat([finalRoot, sibling]));
      } else {
          finalRoot = keccak256(concat([sibling, finalRoot]));
      }
  }

  const signatureBytes = concat(sigParts);

  return {
      message: messageBytes,
      signature: ethers.hexlify(signatureBytes),
      publicKeyRoot: ethers.hexlify(finalRoot)
  };
}
