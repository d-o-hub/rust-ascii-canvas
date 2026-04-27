const fs = require('fs');
const path = require('path');

const filePath = path.join(__dirname, 'web/main.ts');
let content = fs.readFileSync(filePath, 'utf8');

content = content.replace(
`        case 'DrawGrid':
            ctx.strokeStyle = cmd.color as string || '#333333';
            ctx.lineWidth = 0.5;
            ctx.beginPath();

            const panX = cmd.pan_x as number || 0;
            const panY = cmd.pan_y as number || 0;
            const cellW = cmd.cell_width as number || charWidth;
            const cellH = cmd.cell_height as number || lineHeight;`,
`        case 'DrawGrid': {
            ctx.strokeStyle = cmd.color as string || '#333333';
            ctx.lineWidth = 0.5;
            ctx.beginPath();

            const panX = cmd.pan_x as number || 0;
            const panY = cmd.pan_y as number || 0;
            const cellW = cmd.cell_width as number || charWidth;
            const cellH = cmd.cell_height as number || lineHeight;`
);

content = content.replace(
`            // Horizontal lines
            for (let y = panY % cellH; y < canvas.height; y += cellH) {
                ctx.moveTo(0, y);
                ctx.lineTo(canvas.width, y);
            }

            ctx.stroke();
            break;`,
`            // Horizontal lines
            for (let y = panY % cellH; y < canvas.height; y += cellH) {
                ctx.moveTo(0, y);
                ctx.lineTo(canvas.width, y);
            }

            ctx.stroke();
            break;
        }`
);

fs.writeFileSync(filePath, content);
console.log('web/main.ts patched');
