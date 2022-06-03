# Design

<script src="https://unpkg.com/svg-pan-zoom@3.6.1/dist/svg-pan-zoom.min.js" >

</script>

<embed style="border-style: solid; width: 100%" type="image/svg+xml" src="./figures/domain-model.drawio.svg" id="my-embed"/>

<script>
    document.getElementById('my-embed').addEventListener('load', function(){
        let pan = svgPanZoom(document.getElementById('my-embed'), {controlIconsEnabled: true});
        pan.resize();
        pan.pan();
        pan.center();
    })
</script>