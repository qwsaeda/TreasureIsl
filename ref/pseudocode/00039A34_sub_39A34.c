/* ea:   0x00039A34 */
/* name: sub_39A34 */
/* size: 108 */
_BYTE *__fastcall sub_39A34(int a1, char *a2)
{
  _BYTE *v3; // r5
  _BYTE *v4; // r7
  int v5; // r2
  int v6; // r1
  char v7; // r0

  v3 = a2 + 16;
  v4 = sub_343AC(a1, 1);
  if ( v4 != (_BYTE *)-16 )
  {
    v4[16] = *v3;
    v5 = (unsigned __int8)a2[18] | ((unsigned __int8)a2[19] << 8);
    v6 = v5 | ((unsigned __int8)a2[20] << 16) | ((unsigned __int8)a2[21] << 24);
    *((_WORD *)v4 + 9) = v5;
    *((_WORD *)v4 + 10) = HIWORD(v6);
  }
  v7 = *a2;
  *((_DWORD *)v4 + 2) = 0;
  *v4 = v7;
  *((_DWORD *)v4 + 3) = 0;
  return v4;
}

