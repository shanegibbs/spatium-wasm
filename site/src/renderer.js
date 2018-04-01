const gridHeight = 3
const gridWidth = 3
const gridOffsetX = 20
const gridOffsetY = 20
const gridStepHeight = 60
const gridStepWidth = 60

class Renderer {
    constructor(canvas) {
        const ctx = canvas.getContext("2d")

        canvas.width = (gridStepWidth * gridWidth) + (gridOffsetX * 2)
        canvas.style.width = canvas.width + "px"
        canvas.height = (gridStepHeight * gridHeight) + (gridOffsetY * 2)
        canvas.style.height = canvas.height + "px"

        // Draw crisp lines
        // http://www.mobtowers.com/html5-canvas-crisp-lines-every-time/
        ctx.translate(0.5, 0.5)

        this.canvas = canvas
        this.ctx = ctx
    }
    render(renderingInfo) {
        this.clearScreen()
        this.drawSprite(1, 1, 1)
        this.drawSprite(2, 2, 2)
        this.drawSprite(0, renderingInfo.x, renderingInfo.y)
    }
    clearScreen() {
        const canvas = this.canvas
        const ctx = this.ctx

        // clear
        ctx.fillStyle = "white"
        ctx.fillRect(0, 0, canvas.width, canvas.height)

        // draw grid
        ctx.beginPath()
        ctx.moveTo(gridOffsetX, gridOffsetY)
        ctx.lineTo(gridOffsetX, gridOffsetY + (gridStepHeight * gridHeight))
        ctx.lineTo(gridOffsetX + (gridStepWidth * gridWidth), gridOffsetY + (gridStepHeight * gridHeight))
        ctx.lineTo(gridOffsetX + (gridStepWidth * gridWidth), gridOffsetY)
        ctx.lineTo(gridOffsetX, gridOffsetY)
        ctx.strokeStyle = "black"
        ctx.lineWidth = 1

        for (let x = 1; x < gridWidth; x++) {
            ctx.moveTo(gridOffsetX + (gridStepWidth * x), gridOffsetY)
            ctx.lineTo(gridOffsetX + (gridStepWidth * x), gridOffsetY + gridStepHeight * gridHeight)
        }
        for (let y = 1; y < gridHeight; y++) {
            ctx.moveTo(gridOffsetX, gridOffsetY + gridStepHeight * y)
            ctx.lineTo(gridOffsetX + (gridStepWidth * gridWidth), gridOffsetY + gridStepHeight * y)
        }

        ctx.stroke()
    }
    drawSprite(i, x, y) {
        const ctx = this.ctx;

        if (i == 0) {
            ctx.fillStyle = "blue"
        } else if (i == 1) {
            ctx.fillStyle = "black"
        } else if (i == 2) {
            ctx.fillStyle = "green"
        }

        ctx.fillRect(
            gridOffsetX + gridStepWidth * x,
            gridOffsetY + gridStepHeight * y,
            gridStepWidth, gridStepHeight)

        ctx.strokeStyle = "black"
        ctx.lineWidth = 1
        ctx.rect(gridOffsetX + gridStepWidth * x, gridOffsetY + gridStepHeight * y, gridStepWidth, gridStepHeight);
        ctx.stroke()
    }
}

export default Renderer
