in vec4 position;
uniform mat4 projMatrix;
uniform mat4 viewMatrix;
uniform mat4 modelMatrix;
out vec4 testColor;

void main() {
    gl_Position = projMatrix * viewMatrix * modelMatrix * position;
    testColor = (position + 1.0) * 0.5;
}
