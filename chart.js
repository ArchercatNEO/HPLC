export function GenerateChart(data) {
    const series = {
        series: [
            {
                name: 'series-1',
                data: data.filteredData.map(e => Pointify(e[0], e[1])),
            },
            {
                name: 'series-2',
                data: data.baseline.map(e => Pointify(data.times[e], data.values[e]))
            },
            {
                name: 'series-3',
                data: data.peaks.map(e => Pointify(data.times[e], data.values[e]))
            }
        ],
    };
    const options = {
        fullWidth: true,
        axisX: {
            divisor: 5,
            type: Chartist.FixedScaleAxis,
            labelInterpolationFnc: function (value) {
                return value;
            }
        },
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
    const the_chart = new Chartist.Line('#my-chart', series, options);
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
function Pointify(x, y) { return `x:${x},y:${y}`; }
