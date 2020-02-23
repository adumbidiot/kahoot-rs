var _ = {};
_.replace = function (str, regex, f) {
    var string = str + '';
    return string.replace(regex, f);
}

this.angular = {};
this.angular.isDate = function (value) {
    return false;
}
this.angular.isArray = function () {
    return false;
}
this.angular.isObject = function () {
    return false;
}
this.angular.isString = function () {
    return false;
}
