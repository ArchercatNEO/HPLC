"use strict";
function ProcessFile(event, noise) {
    const target = event.target;
    if (target === null || target === undefined) {
        alert("Empty file");
        return;
    }
    const file = target.files[0];
    const reader = new FileReader();
    reader.onload = (event) => Process(event, noise);
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
    const { peaks, gapped } = GeneratePeaks(values);
    const areas = GenerateAreas(values, floor, peaks);
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
function GeneratePeaks(values) {
    const first = Derivative(values);
    const second = Derivative(first);
    const output = {
        peaks: [],
        valleys: [],
        gapped: []
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
    const output = [];
    for (let i = 1; i < peaks.length; i++)
        output.push(AreaBetween(values, floor, peaks[i - 1], peaks[i]));
    return output;
}
function Derivative(array) {
    const output = [array[0], array[1]];
    for (let i = 2; i < array.length; i++)
        output.push((array[i] - array[i - 2]) / 2);
    return output;
}
function AreaBetween(values, floor, start, end) {
    let area = 0;
    for (let i = start; i < end; i++) {
        area += (values[i] - floor[i] + values[i + 1] - floor[i + 1]) / 2;
    }
    return area;
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
