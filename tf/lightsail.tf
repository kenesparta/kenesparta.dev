# ── Lightsail Container Service ──────────────────────────────────────────────
# Runs the site container pulled from the PRIVATE ECR repo (ecr.tf) through the
# service's ECR image-puller role. The blog data lives in a Lightsail managed
# Postgres database (created outside Terraform); the app reaches it via
# DATABASE_URL.

resource "aws_lightsail_container_service" "app" {
  name        = "kenesparta-app"
  power       = "nano" # 0.25 vCPU / 512 MB, ~$7/mo
  scale       = 1
  is_disabled = false

  # ECR image-puller role: lets the service pull the private image
  # (aws_ecr_repository_policy.lightsail_pull grants it on the repo side).
  private_registry_access {
    ecr_image_puller_role {
      is_active = true
    }
  }

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-lightsail"
    }
  )
}

# ── Secrets (sops/age) ───────────────────────────────────────────────────────
# DATABASE_URL points at the Lightsail managed Postgres (created outside
# Terraform). Decrypted at plan/apply time; the value lands in the TF state in
# S3 (encrypted bucket) — same exposure class as the old IAM access key.
data "sops_file" "prod_secrets" {
  source_file = "${path.module}/../secrets/prod.enc.env"
  input_type  = "dotenv"
}

# ── Deployment (container spec: image, env, port, health check) ──────────────
resource "aws_lightsail_container_service_deployment_version" "app" {
  service_name = aws_lightsail_container_service.app.name

  container {
    container_name = "app"
    # CI rollout: terraform apply -target of this resource with
    # TF_VAR_image_version=vX.Y.Z (publish-image.yml / `make rollout`).
    image = "${aws_ecr_repository.app.repository_url}:${var.image_version}"

    ports = {
      "3000" = "HTTP"
    }

    environment = {
      LEPTOS_SITE_ADDR = "0.0.0.0:3000"
      RUST_LOG         = "info"
      DATABASE_URL     = data.sops_file.prod_secrets.data["DATABASE_URL"]
    }
  }

  public_endpoint {
    container_name = "app"
    container_port = 3000

    health_check {
      healthy_threshold   = 2
      unhealthy_threshold = 5
      timeout_seconds     = 5
      interval_seconds    = 10
      path                = "/"
      success_codes       = "200-399"
    }
  }
}

output "lightsail_app_url" {
  value       = aws_lightsail_container_service.app.url
  description = "Default HTTPS domain of the Lightsail container service (for verification and as the CloudFront origin)."
}
