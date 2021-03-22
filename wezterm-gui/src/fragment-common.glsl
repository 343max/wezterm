// This file is automatically prepended to the various -frag shaders.

precision highp float;

in float o_has_color;
in vec2 o_cursor;
in vec2 o_tex;
in vec2 o_underline;
in vec3 o_hsv;
in vec4 o_bg_color;
in vec4 o_cursor_color;
in vec4 o_fg_color;
in vec4 o_underline_color;

out vec4 color;

uniform vec3 foreground_text_hsb;

float multiply_one(float src, float dst, float inv_dst_alpha, float inv_src_alpha) {
  return (src * dst) + (src * (inv_dst_alpha)) + (dst * (inv_src_alpha));
}

// Alpha-regulated multiply to colorize the glyph bitmap.
vec4 multiply(vec4 src, vec4 dst) {
  float inv_src_alpha = 1.0 - src.a;
  float inv_dst_alpha = 1.0 - dst.a;

  return vec4(
      multiply_one(src.r, dst.r, inv_dst_alpha, inv_src_alpha),
      multiply_one(src.g, dst.g, inv_dst_alpha, inv_src_alpha),
      multiply_one(src.b, dst.b, inv_dst_alpha, inv_src_alpha),
      dst.a);
}

vec3 rgb2hsv(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

const vec3 unit3 = vec3(1.0, 1.0, 1.0);

vec4 apply_hsv(vec4 c, vec3 transform)
{
  if (transform == unit3) {
    return c;
  }
  vec3 hsv = rgb2hsv(c.rgb) * transform;
  return vec4(hsv2rgb(hsv).rgb, c.a);
}

vec4 from_linear(vec4 v) {
  return pow(v, vec4(2.2));
}

vec4 to_linear(vec4 v) {
  return pow(v, vec4(1.0/2.2));
}

// Given glyph, the greyscale rgba value computed by freetype,
// and color, the desired color, compute the resultant pixel
// value for rendering over the top of the given background
// color.
//
// The freetype glyph is greyscale (R=G=B=A) when font_antialias=Greyscale,
// where each channel holds the brightness of the pixel.
// It holds separate intensity values for the R, G and B channels when
// subpixel anti-aliasing is in use, with an approximated A value
// derived from the R, G, B values.
//
// In sub-pixel mode we don't want to look at glyph.a as we effective
// have per-channel alpha.  In greyscale mode, glyph.a is the same
// as the other channels, so this routine ignores glyph.a when
// computing the blend, but does include that value for the returned
// alpha value.
//
// See also: https://www.puredevsoftware.com/blog/2019/01/22/sub-pixel-gamma-correct-font-rendering/
vec4 colorize(vec4 glyph, vec4 color, vec4 background) {
  // Why do we linearize the glyph here?
  // I don't think that this is needed, but! without it, the
  // text doesn't render as bold as it used to prior to "fixing"
  // the textures to be properly srgb input and preventing the
  // shader from outputting SRGB directly.
  // The glyph data is populated by the rasterizer and it takes
  // care to convert RGB to SRGB to match the texture format.
  // Assuming that GL is respecting the surface's SRGB encoding,
  // the values we get here should be automatically linearized
  // by this point, so we shouldn't need to linearize them again.
  glyph = to_linear(glyph);

  float r = glyph.r * color.r + (1.0 - glyph.r) * background.r;
  float g = glyph.g * color.g + (1.0 - glyph.g) * background.g;
  float b = glyph.b * color.b + (1.0 - glyph.b) * background.b;

  return vec4(r, g, b, glyph.a);
//  return vec4(glyph.rgb * color.rgb, glyph.a);
}
