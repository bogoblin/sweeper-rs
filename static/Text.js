export class TextStyle {
    textBaseline = 'top';
    font = '20px monospace';
    strokeStyle = 'white';
    lineWidth = 5;
    fillStyle = 'blue';
}

export const BlueText = new TextStyle();

export function drawText(context, text, position, style = BlueText) {
    context.textBaseline = style.textBaseline;
    context.font = style.font;
    context.strokeStyle = style.strokeStyle;
    context.lineWidth = style.lineWidth;
    context.fillStyle = style.fillStyle;
    context.strokeText(text, position[0], position[1]);
    context.fillText(text, position[0], position[1]);
}