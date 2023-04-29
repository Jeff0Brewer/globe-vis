precision highp float;

in vec4 testColor;
out vec4 fragColor;

void main() {
    vec2 cxy = 2.0 * gl_PointCoord - 1.0;
    float radius = dot(cxy, cxy);
    if (radius > 1.0) {
        discard;
    }
    fragColor = vec4(1.0, 1.0, 1.0, 1.0);
}
