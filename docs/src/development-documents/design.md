# Design

<script src="https://unpkg.com/@panzoom/panzoom@4.5.0/dist/panzoom.min.js"></script>

## Domain Model

<div  style="overflow: hidden; border-style: solid; width: 110%">
<img src="./figures/domain-model.drawio.svg" class="diagram" alt="diagram" />
</div>

## Data Model

<div  style="overflow: hidden; border-style: solid; width: 110%">
<img src="./figures/data-model.drawio.svg" class="diagram" alt="diagram" />
</div>

## Crate and Project Structure

<div  style="overflow: hidden; border-style: solid; width: 110%">
<img src="./figures/crate-and-projects-structure.drawio.svg" class="diagram" alt="diagram" />
</div>

<script>
    Array.from(document.getElementsByClassName('diagram')).forEach(e => {
         panzoom = Panzoom(e, {});
         e.parentElement.addEventListener('wheel', panzoom.zoomWithWheel)
    })
</script>
