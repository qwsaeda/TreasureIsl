/* ea:   0x00061598 */
/* name: sub_61598 */
/* size: 60 */
_DWORD *__fastcall sub_61598(int a1, _BYTE *a2)
{
  _DWORD *v3; // r5

  v3 = sub_60518(a1, 1);
  if ( v3 != (_DWORD *)-16 )
    sub_619C8(v3 + 4, a2 + 16);
  *(_BYTE *)v3 = *a2;
  v3[2] = 0;
  v3[3] = 0;
  return v3;
}

