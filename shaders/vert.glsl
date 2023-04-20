attribute vec4 position;
uniform mat4 viewProjMatrix;
uniform mat4 modelMatrix;
varying vec4 testColor;

void main() {
    gl_Position = viewProjMatrix * modelMatrix * position;
    testColor = (position + 1.0) * 0.5;
}
