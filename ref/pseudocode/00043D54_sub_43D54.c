/* ea:   0x00043D54 */
/* name: sub_43D54 */
/* size: 64 */
int __fastcall sub_43D54(_DWORD *a1, _DWORD *a2)
{
  memcpy(a1 + 124, a2 + 124, 0x1E0u);
  memcpy(a1 + 2, a2 + 2, 0x1E0u);
  a1[123] = a2[123];
  a1[253] = 0;
  return 0;
}

