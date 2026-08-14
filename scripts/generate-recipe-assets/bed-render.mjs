/**
 * 床物品 3D 立体图标渲染器
 * 依据 26.2 官方床模型几何（template_bed_head/foot）与分面贴图，
 * 软件光栅化（painter's + z-buffer + 超采样）渲染 16x16 GUI 立体床图标。
 */
import { PNG } from "pngjs";

const DEG = Math.PI / 180;

function mulMat(a, b) {
  return a.map((row) =>
    [0, 1, 2].map(
      (j) => row[0] * b[0][j] + row[1] * b[1][j] + row[2] * b[2][j],
    ),
  );
}
function matRot(ax, ay, az) {
  const ca = Math.cos(ax * DEG),
    sa = Math.sin(ax * DEG);
  const cb = Math.cos(ay * DEG),
    sb = Math.sin(ay * DEG);
  const cc = Math.cos(az * DEG),
    sc = Math.sin(az * DEG);
  const rz = [
    [cc, -sc, 0],
    [sc, cc, 0],
    [0, 0, 1],
  ];
  const ry = [
    [cb, 0, sb],
    [0, 1, 0],
    [-sb, 0, cb],
  ];
  const rx = [
    [1, 0, 0],
    [0, ca, -sa],
    [0, sa, ca],
  ];
  return mulMat(mulMat(rx, ry), rz);
}
function applyM(m, v) {
  return [
    m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
    m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
    m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
  ];
}

const FACE_NORMAL = {
  up: [0, 1, 0],
  down: [0, -1, 0],
  north: [0, 0, -1],
  south: [0, 0, 1],
  east: [1, 0, 0],
  west: [-1, 0, 0],
};

// 床头在 z=0 端（north 为床头高板），模型 UV 直接取 26.2 template_bed_head.json
const HEAD_ELEMENTS = [
  {
    from: [0, 3, 0],
    to: [16, 9, 16],
    faces: {
      down: { uv: [0, 0, 16, 16], tex: "bed_down" },
      up: { uv: [0, 0, 16, 16], tex: "head_up" },
      north: { uv: [0, 7, 16, 13], tex: "bed_head_north" },
      west: { uv: [0, 7, 16, 13], tex: "head_west" },
      east: { uv: [0, 7, 16, 13], tex: "head_east" },
    },
  },
  {
    from: [0, 0, 0],
    to: [3, 3, 3],
    faces: {
      down: { uv: [6, 13, 9, 16], tex: "head_west" },
      north: { uv: [13, 13, 16, 16], tex: "bed_head_north" },
      south: { uv: [3, 13, 6, 16], tex: "head_west" },
      west: { uv: [0, 13, 3, 16], tex: "head_west" },
      east: { uv: [10, 13, 13, 16], tex: "bed_head_north" },
    },
  },
  {
    from: [13, 0, 0],
    to: [16, 3, 3],
    faces: {
      down: { uv: [7, 13, 10, 16], tex: "head_east" },
      north: { uv: [0, 13, 3, 16], tex: "bed_head_north" },
      south: { uv: [10, 13, 13, 16], tex: "head_east" },
      west: { uv: [3, 13, 6, 16], tex: "bed_head_north" },
      east: { uv: [13, 13, 16, 16], tex: "head_east" },
    },
  },
];

// 床尾在 z 高端（south 为床尾低板），items/white_bed.json 中 foot 平移 [0,0,1]
const FOOT_ELEMENTS = [
  {
    from: [0, 3, 0],
    to: [16, 9, 16],
    faces: {
      down: { uv: [0, 0, 16, 16], tex: "bed_down" },
      up: { uv: [0, 0, 16, 16], tex: "foot_up" },
      south: { uv: [0, 7, 16, 13], tex: "foot_south" },
      west: { uv: [0, 7, 16, 13], tex: "foot_west" },
      east: { uv: [0, 7, 16, 13], tex: "foot_east" },
    },
  },
  {
    from: [0, 0, 13],
    to: [3, 3, 16],
    faces: {
      down: { uv: [7, 13, 10, 16], tex: "foot_west" },
      north: { uv: [10, 13, 13, 16], tex: "foot_west" },
      south: { uv: [0, 13, 3, 16], tex: "foot_south" },
      west: { uv: [13, 13, 16, 16], tex: "foot_west" },
      east: { uv: [3, 13, 6, 16], tex: "foot_south" },
    },
  },
  {
    from: [13, 0, 13],
    to: [16, 3, 16],
    faces: {
      down: { uv: [6, 13, 9, 16], tex: "foot_east" },
      north: { uv: [3, 13, 6, 16], tex: "foot_east" },
      south: { uv: [13, 13, 16, 16], tex: "foot_south" },
      west: { uv: [10, 13, 13, 16], tex: "foot_south" },
      east: { uv: [0, 13, 3, 16], tex: "foot_east" },
    },
  },
];

function quadForFace(el, faceName) {
  const [fx, fy, fz] = el.from;
  const [tx, ty, tz] = el.to;
  const [u1, v1, u2, v2] = el.faces[faceName].uv;
  return {
    north: [
      [fx, ty, fz, u1, v1],
      [tx, ty, fz, u2, v1],
      [tx, fy, fz, u2, v2],
      [fx, fy, fz, u1, v2],
    ],
    south: [
      [fx, ty, tz, u1, v1],
      [tx, ty, tz, u2, v1],
      [tx, fy, tz, u2, v2],
      [fx, fy, tz, u1, v2],
    ],
    east: [
      [tx, ty, fz, u1, v1],
      [tx, ty, tz, u2, v1],
      [tx, fy, tz, u2, v2],
      [tx, fy, fz, u1, v2],
    ],
    west: [
      [fx, ty, tz, u1, v1],
      [fx, ty, fz, u2, v1],
      [fx, fy, fz, u2, v2],
      [fx, fy, tz, u1, v2],
    ],
    up: [
      [fx, ty, fz, u1, v2],
      [tx, ty, fz, u2, v2],
      [tx, ty, tz, u2, v1],
      [fx, ty, tz, u1, v1],
    ],
    down: [
      [fx, fy, tz, u1, v2],
      [tx, fy, tz, u2, v2],
      [tx, fy, fz, u2, v1],
      [fx, fy, fz, u1, v1],
    ],
  }[faceName];
}

// 物品栏 GUI：display.gui rotation [30,340,0] / translation [2,3,0] / scale [0.5325]
const ROT = matRot(30, 340, 0);
const SCALE = 0.5325;
const TRANS = [2, 3, 0];

function transformVertex(v) {
  const s = [v[0] * SCALE, v[1] * SCALE, v[2] * SCALE];
  const r = applyM(ROT, s);
  return [r[0] + TRANS[0], r[1] + TRANS[1], r[2] + TRANS[2]];
}

function sampleTex(png, u, v) {
  const w = png.width,
    h = png.height;
  const fx = Math.min(Math.max((u / 16) * w, 0), w - 1);
  const fy = Math.min(Math.max((v / 16) * h, 0), h - 1);
  const x0 = Math.floor(fx),
    y0 = Math.floor(fy);
  const x1 = Math.min(x0 + 1, w - 1),
    y1 = Math.min(y0 + 1, h - 1);
  const dx = fx - x0,
    dy = fy - y0;
  const px = (xx, yy) => {
    const i = (yy * w + xx) * 4;
    return [png.data[i], png.data[i + 1], png.data[i + 2], png.data[i + 3]];
  };
  const c00 = px(x0, y0),
    c10 = px(x1, y0),
    c01 = px(x0, y1),
    c11 = px(x1, y1);
  return [0, 1, 2, 3].map(
    (k) =>
      c00[k] * (1 - dx) * (1 - dy) +
      c10[k] * dx * (1 - dy) +
      c01[k] * (1 - dx) * dy +
      c11[k] * dx * dy,
  );
}

function downsample(render, SS) {
  const OUT = 16;
  const out = new PNG({ width: OUT, height: OUT });
  for (let y = 0; y < OUT; y += 1) {
    for (let x = 0; x < OUT; x += 1) {
      const x0 = x * SS,
        y0 = y * SS;
      let r = 0,
        g = 0,
        b = 0,
        a = 0,
        n = 0;
      for (let yy = y0; yy < y0 + SS; yy += 1) {
        for (let xx = x0; xx < x0 + SS; xx += 1) {
          const i = (yy * render.width + xx) * 4;
          r += render.data[i];
          g += render.data[i + 1];
          b += render.data[i + 2];
          a += render.data[i + 3];
          n += 1;
        }
      }
      const o = (y * OUT + x) * 4;
      out.data[o] = Math.round(r / n);
      out.data[o + 1] = Math.round(g / n);
      out.data[o + 2] = Math.round(b / n);
      out.data[o + 3] = Math.round(a / n);
    }
  }
  return out;
}

/**
 * 渲染 16x16 床图标。相机在 +z 侧看向 -z（床顶面可见），屏幕 y 向下翻转。
 * 模型经 display.gui 变换后按 bbox 居中并缩放填满 16x16（同原版 RenderItem 适配）。
 * @param {Object<string, PNG>} texs 9 张分面贴图（bed_head_north / bed_down / head 系 / foot 系）
 * @returns {PNG} 16x16 立体床图标
 */
export function renderBedIcon(texs) {
  const model = [...HEAD_ELEMENTS.map((el) => JSON.parse(JSON.stringify(el)))];
  for (const el of JSON.parse(JSON.stringify(FOOT_ELEMENTS))) {
    el.from[2] += 1;
    el.to[2] += 1;
    model.push(el);
  }
  const verts = [];
  for (const el of model) {
    for (const faceName of Object.keys(el.faces)) {
      for (const [x, y, z] of quadForFace(el, faceName))
        verts.push(transformVertex([x, y, z]));
    }
  }
  const minX = Math.min(...verts.map((p) => p[0])),
    maxX = Math.max(...verts.map((p) => p[0]));
  const minY = Math.min(...verts.map((p) => p[1])),
    maxY = Math.max(...verts.map((p) => p[1]));
  const fit = 16 / Math.max(maxX - minX, maxY - minY);
  const cx = (minX + maxX) / 2,
    cy = (minY + maxY) / 2;
  const toPx = (x, y) => [8 + (x - cx) * fit, 8 - (y - cy) * fit];

  const SS = 4;
  const wPx = 16 * SS;
  const render = new PNG({ width: wPx, height: wPx });
  const zbuf = new Float32Array(wPx * wPx).fill(-Infinity);

  const faces = [];
  for (const el of model) {
    for (const [faceName, face] of Object.entries(el.faces)) {
      const n = applyM(ROT, FACE_NORMAL[faceName]);
      if (n[2] * -1 >= 0) continue;
      const quad = quadForFace(el, faceName);
      const pts = quad.map(([x, y, z, u, v]) => {
        const p = transformVertex([x, y, z]);
        const [sx, sy] = toPx(p[0], p[1]);
        return { sx: sx * SS, sy: sy * SS, z: p[2], u, v };
      });
      faces.push({ tex: texs[face.tex], pts });
    }
  }
  faces.sort((a, b) => {
    const za = a.pts.reduce((s, p) => s + p.z, 0) / 4;
    const zb = b.pts.reduce((s, p) => s + p.z, 0) / 4;
    return za - zb;
  });

  const edgeFn = (p0, p1, x, y) =>
    (x - p0.sx) * (p1.sy - p0.sy) - (y - p0.sy) * (p1.sx - p0.sx);
  for (const face of faces) {
    for (const tri of [
      [0, 1, 2],
      [0, 2, 3],
    ]) {
      const [i0, i1, i2] = tri;
      const a = face.pts[i0],
        b = face.pts[i1],
        c = face.pts[i2];
      const minTx = Math.max(0, Math.floor(Math.min(a.sx, b.sx, c.sx)));
      const maxTx = Math.min(wPx - 1, Math.ceil(Math.max(a.sx, b.sx, c.sx)));
      const minTy = Math.max(0, Math.floor(Math.min(a.sy, b.sy, c.sy)));
      const maxTy = Math.min(wPx - 1, Math.ceil(Math.max(a.sy, b.sy, c.sy)));
      const area = edgeFn(a, b, c.sx, c.sy);
      if (Math.abs(area) < 1e-9) continue;
      for (let py = minTy; py <= maxTy; py += 1) {
        const sy = py + 0.5;
        for (let px = minTx; px <= maxTx; px += 1) {
          const sx = px + 0.5;
          const w0 = edgeFn(b, c, sx, sy);
          const w1 = edgeFn(c, a, sx, sy);
          const w2 = edgeFn(a, b, sx, sy);
          if (
            area > 0 ? w0 < 0 || w1 < 0 || w2 < 0 : w0 > 0 || w1 > 0 || w2 > 0
          )
            continue;
          const inv = 1 / area;
          const l0 = w0 * inv,
            l1 = w1 * inv,
            l2 = w2 * inv;
          const z = a.z * l0 + b.z * l1 + c.z * l2;
          const idx = py * wPx + px;
          if (z <= zbuf[idx]) continue;
          const u = a.u * l0 + b.u * l1 + c.u * l2;
          const v = a.v * l0 + b.v * l1 + c.v * l2;
          const rgba = sampleTex(face.tex, u, v);
          if (rgba[3] < 4) continue;
          zbuf[idx] = z;
          const o = idx * 4;
          render.data[o] = rgba[0];
          render.data[o + 1] = rgba[1];
          render.data[o + 2] = rgba[2];
          render.data[o + 3] = 255;
        }
      }
    }
  }
  return downsample(render, SS);
}
