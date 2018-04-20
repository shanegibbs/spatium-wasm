const gridHeight = 3
const gridWidth = 3
const gridOffsetX = 20
const gridOffsetY = 20

class Renderer {
    constructor(canvas) {
        const ctx = canvas.getContext("2d")

        canvas.width = 240
        canvas.style.width = canvas.width + "px"
        canvas.height = 240
        canvas.style.height = canvas.height + "px"

        // Draw crisp lines
        // http://www.mobtowers.com/html5-canvas-crisp-lines-every-time/
        // ctx.translate(0.5, 0.5)

        this.canvas = canvas
        this.ctx = ctx
    }
    GridStepHeight() {
        return (this.canvas.height - (gridOffsetX * 2)) / this.renderingInfo.height
    }
    GridStepWidth() {
        return (this.canvas.width - (gridOffsetY * 2)) / this.renderingInfo.width
    }
    render(renderingInfo) {
        this.renderingInfo = renderingInfo
        this.clearScreen()
        for (const layer of renderingInfo.layers) {
            let id = 0
            if (layer.name == "agent") {
                id = 0
            } else if (layer.name == "block") {
                id = 1
            } else if (layer.name == "food") {
                id = 2
            }
            for (const point of layer.points) {
                this.drawSprite(id, point.x, point.y)
            }
        }
    }
    clearScreen() {
        const canvas = this.canvas
        const ctx = this.ctx

        // clear
        ctx.fillStyle = "white"
        ctx.fillRect(0, 0, canvas.width, canvas.height)

        if (typeof (this.renderingInfo) == 'undefined') {
            return
        }

        const gridHeight = this.renderingInfo.height
        const gridWidth = this.renderingInfo.width
        const gridStepHeight = this.GridStepHeight()
        const gridStepWidth = this.GridStepWidth()

        // draw grid
        ctx.beginPath()
        ctx.moveTo(gridOffsetX, gridOffsetY)
        ctx.lineTo(gridOffsetX, gridOffsetY + (gridStepHeight * gridHeight))
        ctx.lineTo(gridOffsetX + (gridStepWidth * gridWidth), gridOffsetY + (gridStepHeight * gridHeight))
        ctx.lineTo(gridOffsetX + (gridStepWidth * gridWidth), gridOffsetY)
        ctx.lineTo(gridOffsetX, gridOffsetY)
        ctx.strokeStyle = "black"
        ctx.lineWidth = 1

        if (gridWidth < 10 && gridHeight < 10) {
            for (let x = 1; x < gridWidth; x++) {
                ctx.moveTo(gridOffsetX + (gridStepWidth * x), gridOffsetY)
                ctx.lineTo(gridOffsetX + (gridStepWidth * x), gridOffsetY + gridStepHeight * gridHeight)
            }
            for (let y = 1; y < gridHeight; y++) {
                ctx.moveTo(gridOffsetX, gridOffsetY + gridStepHeight * y)
                ctx.lineTo(gridOffsetX + (gridStepWidth * gridWidth), gridOffsetY + gridStepHeight * y)
            }
        }

        ctx.stroke()
    }
    drawSprite(i, x, y) {
        const ctx = this.ctx;
        const gridHeight = this.renderingInfo.height
        const gridWidth = this.renderingInfo.width

        if (i == 0) {
            ctx.fillStyle = "blue"
        } else if (i == 1) {
            ctx.fillStyle = "black"
        } else if (i == 2) {
            ctx.fillStyle = "green"
        }
        ctx.strokeStyle = ctx.fillStyle

        let gridStepHeight = this.GridStepHeight()
        let gridStepWidth = this.GridStepWidth()

        ctx.fillRect(
            gridOffsetX + gridStepWidth * x,
            gridOffsetY + gridStepHeight * y,
            gridStepWidth, gridStepHeight)

        if (gridWidth < 10 && gridHeight < 10) {
            ctx.strokeStyle = "black"
            ctx.lineWidth = 1
            ctx.rect(gridOffsetX + gridStepWidth * x, gridOffsetY + gridStepHeight * y, gridStepWidth, gridStepHeight);
        }
        ctx.stroke()
    }
}

export default Renderer
