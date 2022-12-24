// Code generated by @apexlang/codegen. DO NOT EDIT.

package http

import (
	"github.com/nanobus/nanobus/pkg/runtime"
	"github.com/nanobus/nanobus/pkg/transport"
)

type HttpServerV1Config struct {
	Address    string              `json:"address" yaml:"address" msgpack:"address" mapstructure:"address" validate:"required"`
	Middleware []runtime.Component `json:"middleware,omitempty" yaml:"middleware,omitempty" msgpack:"middleware,omitempty" mapstructure:"middleware"`
	Routers    []runtime.Component `json:"routers,omitempty" yaml:"routers,omitempty" msgpack:"routers,omitempty" mapstructure:"routers"`
}

func HttpServerV1() (string, transport.Loader) {
	return "nanobus.transport.http.server/v1", HttpServerV1Loader
}
