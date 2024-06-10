<?php

namespace YourVendor\RocksDB;

use Illuminate\Support\ServiceProvider;

class RocksDBServiceProvider extends ServiceProvider {
    /**
     * Perform post-registration booting of services.
     *
     * @return void
     */
    public function boot()
    {
        // Publishes configuration.
        $this->publishes([
            __DIR__.'/../config/rocksdb.php' => config_path('rocksdb.php'),
        ], 'config');
    }

    /**
     * Register bindings in the container.
     *
     * @return void
     */
    public function register()
    {
        // Merges user and package configuration.
        $this->mergeConfigFrom(
            __DIR__.'/../config/rocksdb.php', 'rocksdb'
        );

        // Registers the main class to use with the facade.
        $this->app->singleton('rocksdb', function ($app) {
            return new RocksDBClient(
                config('rocksdb.host'),
                config('rocksdb.port'),
                config('rocksdb.token')
            );
        });
    }
}
