/* ea:   0x000399E0 */
/* name: sub_399E0 */
/* size: 84 */
_BYTE *__fastcall sub_399E0(int a1, _BYTE *a2)
{
  _BYTE *v3; // r6
  int v4; // r2
  int v5; // r1

  v3 = sub_343AC(a1, 1);
  if ( v3 != (_BYTE *)-16 )
  {
    v3[16] = *a2;
    v4 = (unsigned __int8)a2[2] | ((unsigned __int8)a2[3] << 8);
    v5 = v4 | ((unsigned __int8)a2[4] << 16) | ((unsigned __int8)a2[5] << 24);
    *((_WORD *)v3 + 9) = v4;
    *((_WORD *)v3 + 10) = HIWORD(v5);
  }
  return v3;
}

