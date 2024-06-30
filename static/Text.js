class TextStyle {
    textBaseline = 'top';
    font = '20px monospace';
    strokeStyle = 'white';
    lineWidth = 5;
    fillStyle = 'blue';
}

const BlueText = new TextStyle();

function drawText(context, text, position) {
    const style = BlueText;
    context.textBaseline = style.textBaseline;
    context.font = style.font;
    context.strokeStyle = style.strokeStyle;
    context.lineWidth = style.lineWidth;
    context.fillStyle = style.fillStyle;
    context.strokeText(text, position[0], position[1]);
    context.fillText(text, position[0], position[1]);
}