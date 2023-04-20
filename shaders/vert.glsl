attribute vec4 position;
uniform mat4 mvp;
varying vec4 testColor;

void main() {
    gl_Position = mvp * position;
    testColor = (position + 1.0) * 0.5;
}
