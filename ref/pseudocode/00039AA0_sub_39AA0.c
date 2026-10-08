/* ea:   0x00039AA0 */
/* name: sub_39AA0 */
/* size: 60 */
_DWORD *__fastcall sub_39AA0(int a1, _DWORD *a2)
{
  _DWORD *v3; // r0
  _DWORD *v4; // r5

  v3 = sub_34438(a1, 1);
  v4 = v3;
  if ( v3 != (_DWORD *)-16 )
  {
    v3[4] = *a2;
    memcpy(v3 + 5, a2 + 1, 6u);
  }
  return v4;
}

