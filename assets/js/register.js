document.addEventListener('DOMContentLoaded', () => {
  return; // Event has ended — all submissions disabled

  const soloForm = document.getElementById('soloFormEl');
  const otpModalWrapper = document.getElementById('otpModalWrapper');
  const otpForm = document.getElementById('otpFormEl');
  const resendBtn = document.getElementById('resendOtpBtn');

  let currentRegId = null;

  function showOtpModal() {
    otpModalWrapper.classList.remove('hidden');
  }

  if (soloForm) {
    soloForm.addEventListener('submit', async (e) => {
      e.preventDefault();
      const submitError = document.getElementById('soloSubmitError');
      submitError.classList.add('hidden');

      const submitButton = soloForm.querySelector('button[type="submit"]');
      submitButton.disabled = true;
      submitButton.textContent = 'REGISTERING...';

      const payload = {
        team_name: document.getElementById('soloCallsign').value,
        registration_type: 'single',
        leader: {
          name: document.getElementById('soloFullName').value,
          gender: document.getElementById('soloGender').value.toLowerCase(),
          email: document.getElementById('soloEmail').value,
          mobile: {
            cc: document.getElementById('soloPhoneCC').value,
            number: document.getElementById('soloPhone').value
          },
          college: document.getElementById('soloCollegeName').value,
          degree: document.getElementById('soloDegree').value,
          department: document.getElementById('soloDepartment').value,
          year: parseInt(document.getElementById('soloYear').value, 10),
          location: document.getElementById('soloLocation').value
        },
        member: null
      };

      try {
        const enc = await encryptPayload(payload, '/_/x/register');
        const response = await fetch(API_ENDPOINTS.register, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ payload: enc })
        });

        let data;
        try {
          data = await response.json();
        } catch {
          throw new Error(`Server returned non-JSON response (${response.status}).`);
        }

        if (!response.ok) {
          throw new Error(data.error?.msg || data.msg || 'Registration failed');
        }

        currentRegId = data.data.team_id;
        showOtpModal();

      } catch (error) {
        submitError.textContent = error.message;
        submitError.classList.remove('hidden');
      } finally {
        submitButton.disabled = false;
        submitButton.textContent = 'I\'M READY TO HACK';
      }
    });
  }

  if (otpForm) {
    otpForm.addEventListener('submit', async (e) => {
      e.preventDefault();
      const submitError = document.getElementById('otpSubmitError');
      submitError.classList.add('hidden');

      const submitButton = otpForm.querySelector('button[type="submit"]');
      submitButton.disabled = true;
      submitButton.textContent = 'VERIFYING...';

      const payload = {
        team_id: currentRegId,
        otp: {
          leader: document.getElementById('leaderOtp').value,
          member: null
        }
      };

      try {
        const enc = await encryptPayload(payload, '/_/x/authn/verify');
        const response = await fetch(API_ENDPOINTS.authn.verify, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ payload: enc })
        });

        let data;
        try {
          data = await response.json();
        } catch {
          throw new Error(`Server returned non-JSON response (${response.status}).`);
        }

        if (!response.ok) {
          throw new Error(data.error?.msg || data.msg || 'OTP Verification failed');
        }

        submitButton.textContent = 'INITIATING PAYMENT...';

        const pmtPayload = {
          team_id: currentRegId,
          return_url: window.location.origin + '/payment/status?team_id=' + currentRegId
        };
        const pmtEnc = await encryptPayload(pmtPayload, '/_/x/pmt/create');
        const pmtResponse = await fetch(API_ENDPOINTS.payment.create, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ payload: pmtEnc })
        });

        let pmtData;
        try {
          pmtData = await pmtResponse.json();
        } catch {
          throw new Error(`Payment server returned non-JSON response (${pmtResponse.status}).`);
        }

        if (!pmtResponse.ok) {
          throw new Error(pmtData.error?.msg || pmtData.msg || 'Payment initiation failed');
        }

        window.location.href = pmtData.data.payment_link;

      } catch (error) {
        submitError.textContent = error.message;
        submitError.classList.remove('hidden');
      } finally {
        submitButton.disabled = false;
        submitButton.textContent = 'VERIFY_UPLINK';
      }
    });
  }

  if (resendBtn) {
    resendBtn.addEventListener('click', async (e) => {
      e.preventDefault();
      const feedback = document.getElementById('resendFeedback');
      resendBtn.disabled = true;
      resendBtn.textContent = '[ Sending... ]';

      try {
        const enc = await encryptPayload({ team_id: currentRegId }, '/_/x/authn/resend');
        const response = await fetch(API_ENDPOINTS.authn.resend, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ payload: enc })
        });

        let data;
        try {
          data = await response.json();
        } catch {
          throw new Error(`Server returned non-JSON response (${response.status}).`);
        }

        if (!response.ok) {
          throw new Error(data.error?.msg || data.msg || 'Failed to resend');
        }

        feedback.textContent = 'Codes Resent Successfully.';
        feedback.classList.remove('hidden');
        feedback.classList.remove('text-red-400');
        feedback.classList.add('text-green-400');

      } catch (error) {
        feedback.textContent = error.message;
        feedback.classList.remove('hidden');
        feedback.classList.remove('text-green-400');
        feedback.classList.add('text-red-400');
      } finally {
        resendBtn.textContent = '[ Resend Codes ]';
        setTimeout(() => {
          resendBtn.disabled = false;
          feedback.classList.add('hidden');
        }, 3000);
      }
    });
  }
});
