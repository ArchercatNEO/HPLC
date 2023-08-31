import { GenerateChart } from "./chart";
export function ProcessFile(event, noise) {
    const reader = new FileReader();
    reader.onload = (event) => Process(event, parseFloat(noise));
    let file;
    if (event instanceof File)
        file = event;
    else {
        const target = event.target;
        if (target === null || target === undefined) {
            alert("Empty file");
            return;
        }
        file = target.files[0];
    }
    reader.readAsText(file, "UTF-8");
}
function Process(event, noise) {
    var _a;
    const raw = (_a = event.target) === null || _a === void 0 ? void 0 : _a.result;
    if (raw === null || raw === undefined) {
        alert("File is empty or invalid");
        return;
    }
    const byRow = raw.split("\r\n");
    // Fill the data to be an array of arrays where each inner array is [time, value]
    const fullData = byRow.map(e => e.split("\t"));
    const numberData = fullData.map(e => e.map(d => parseFloat(d)));
    // Filter the times to only have data that happened between 8.85 and 36 time
    const filteredData = numberData.filter(e => 8.85 < e[0] && e[0] < 36);
    const times = filteredData.map(e => e[0]);
    const values = filteredData.map(e => e[1]);
    const labels = times.map(e => e % 150 === 0 ? e : "");
    const baseline = GenerateBaseline(values);
    const floor = Interpolate(baseline, values);
    const { peaks, gapped, first, second, valleys } = GeneratePeaks(values, noise);
    const { areas, ranges } = GenerateAreas(values, floor, valleys);
    const table = [];
    for (let i = 0; i < areas.length; i++)
        table.push([ranges[i].map(e => times[e]).join("-"), areas[i], times[peaks[i]]].join(", "));
    document.getElementById("heya").innerHTML = table.join("<br>");
    const data = {
        filteredData: filteredData,
        times: times,
        values: values,
        areas: areas,
        baseline: baseline,
        floor: floor,
        peaks: peaks,
        gapped: gapped,
        labels: labels,
        first: first,
        second: second,
    };
    GenerateChart(data);
}
/**
 * Use gradients to draw the tightest possible line that goes under every point
 * @param array the data
 * @returns
 */
function GenerateBaseline(array) {
    const output = [0];
    while (output[output.length - 1] + 1 < array.length) {
        let gradient = Number.MAX_SAFE_INTEGER;
        let best = output[output.length - 1];
        // esto es para que no haya puntos consecutivos, 
        // lo forzamos a que sean al menos 10 puntos entre minimos
        const end = output[output.length - 1] + 5;
        for (let i = end + 1; i < array.length; i++) {
            let grad_i = (array[i] - array[end]) / (i - end);
            if (grad_i < gradient) {
                gradient = grad_i;
                best = i;
            }
        }
        output.push(best);
    }
    return output;
}
function GeneratePeaks(values, noise) {
    const first = Derivative(values);
    const second = Derivative(first);
    const first0 = [];
    for (let i = 1; i < first.length; i++) {
        first0.push(first[i - 1] * first[i] <= -noise ? values[i - 1] : null);
    }
    const second0 = [];
    for (let i = 0; i < second.length; i++) {
        second0.push(second[i - 1] * second[i] <= -noise ? values[i - 1] : null);
    }
    const output = {
        peaks: [],
        valleys: [],
        gapped: [],
        first: first0,
        second: second0
    };
    for (let i = 1; i < values.length; i++) {
        // While the graph is going down keep searching
        while (values[i - 1] > values[i])
            i++;
        if (i >= values.length)
            break;
        // Found minimum
        output.valleys.push(i);
        output.gapped[i] = values[i];
        // While the graph is going up keep searching
        while (values[i - 1] < values[i])
            i++;
        if (i >= values.length)
            break;
        // Found maximum
        output.peaks.push(i);
        output.gapped[i] = values[i];
    }
    return output;
}
function GenerateAreas(values, floor, peaks) {
    const ranges = [];
    const output = [];
    for (let i = 1; i < peaks.length; i++) {
        ranges.push([peaks[i - 1], peaks[i]]);
        output.push(AreaBetween(values, floor, peaks[i - 1], peaks[i]));
    }
    return {
        ranges: ranges,
        areas: output
    };
}
function AreaBetween(values, floor, start, end) {
    let area = 0;
    for (let i = start; i < end; i++) {
        area += (values[i] - floor[i] + values[i + 1] - floor[i + 1]) / 2;
    }
    return area;
}
function Derivative(array) {
    const output = [array[0], array[1]];
    for (let i = 2; i < array.length; i++)
        output.push((array[i] - array[i - 2]) / 2);
    return output;
}
/**
 * Use Interpolation and gradients to draw a straight line between every point in indexArray using valuesArray
*/
function Interpolate(indexArray, valuesArray) {
    const output = [];
    for (let i = 1; i < indexArray.length; i++) {
        // Left and right points 
        let left = indexArray[i - 1];
        let right = indexArray[i];
        let gradient = (valuesArray[right] - valuesArray[left]) / (right - left);
        // Interpolate between the left and right points based on the gradient
        for (let x = 0; x < right - left; x++)
            output.push(gradient * x + valuesArray[left]);
    }
    return output;
}
