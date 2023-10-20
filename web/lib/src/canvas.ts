export const preventDefaultTouchActions = () => {
    document.body.querySelectorAll("canvas").forEach(canvas => {
        canvas.addEventListener("touchstart", e => e.preventDefault())
        canvas.addEventListener("touchmove", e => e.preventDefault())
        canvas.addEventListener("contextmenu", e => e.preventDefault())
    })
}
