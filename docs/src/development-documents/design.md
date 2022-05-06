# Design

<script crossorigin="anonymous" src="https://unpkg.com/svg-pan-zoom@3.6.1/dist/svg-pan-zoom.min.js" ></script>

<embed style="border-style: solid; width: 100%" type="image/svg+xml" src="./figures/domain-model.drawio.svg" class="diagram"/>
<embed style="border-style: solid; width: 100%" type="image/svg+xml" src="./figures/data-model.drawio.svg" class="diagram"/>

<script>
    document.getElementsByClassName('diagram').addEventListener('load', function(){
        let pan = svgPanZoom(document.getElementById('my-embed'), {controlIconsEnabled: true});
        pan.resize();
        pan.pan();
        pan.center();
    })
</script>