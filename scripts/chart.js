"use strict";
function GenerateChart(data) {
    const plots = {
        series: [
            {
                name: 'series-1',
                data: data.values,
            },
            {
                name: 'series-2',
                data: data.floor,
            },
            {
                name: 'series-3',
                data: data.gapped
            }
        ],
    };
    const options = {
        fullWidth: true,
        height: "500px",
        series: {
            'series-1': {
                showLine: true,
                showPoint: false
            },
            'series-2': {
                showPoint: false
            },
            'series-3': {
                showLine: false,
                showPoint: true
            }
        }
    };
    /* Initialize the chart with the above settings */
    const the_chart = new Chartist.Line('#my-chart', plots, options);
    // Listening for draw events that get emitted by the Chartist chart
    the_chart.on('draw', function (data) {
        // If the draw event was triggered from drawing a point on the line chart
        if (data.type === 'point') {
            // We are creating a new path SVG element that draws a triangle around the point coordinates
            const triangle = new Chartist.Svg('path', {
                d: ['M',
                    data.x,
                    data.y - 5,
                    'L',
                    data.x - 5,
                    data.y + 3,
                    'L',
                    data.x + 5,
                    data.y + 3,
                    'z'].join(' '),
                style: 'fill-opacity: 1'
            }, 'ct-area');
            // With data.element we get the Chartist SVG wrapper and we can replace the original point drawn by Chartist with our newly created triangle
            data.element.replace(triangle);
        }
    });
}
