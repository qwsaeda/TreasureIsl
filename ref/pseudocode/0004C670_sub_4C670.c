/* ea:   0x0004C670 */
/* name: sub_4C670 */
/* size: 368 */
BOOL __fastcall sub_4C670(int a1, int a2, char a3)
{
  unsigned __int16 *v5; // r5
  unsigned __int16 *v6; // r0
  _WORD *v7; // r4
  unsigned int v8; // r3
  unsigned int v9; // r2
  unsigned int v10; // r2
  _WORD *v11; // r4
  unsigned int v12; // r3
  unsigned int v13; // r2

  v5 = (unsigned __int16 *)(a1 + 136);
  v6 = (unsigned __int16 *)(a1 + 136);
  if ( a3 != 0 )
  {
    v11 = (_WORD *)(a1 + 140);
    v12 = (unsigned __int16)(sub_16C9C(v6, 1) - *(_WORD *)(a2 + 34));
    v13 = *(unsigned __int16 *)(a1 + 140);
    if ( v12 < v13 )
      *v11 = v13 - v12;
    else
      *v11 = 0;
    LOWORD(v10) = sub_16C9C(v5, 0) - *(_WORD *)(a2 + 32);
  }
  else
  {
    v7 = (_WORD *)(a1 + 140);
    v8 = (unsigned __int16)(sub_16C9C(v6, 1) / 5u - *(_WORD *)(a2 + 40));
    v9 = *(unsigned __int16 *)(a1 + 140);
    if ( v8 < v9 )
      *v7 = v9 - v8;
    else
      *v7 = 0;
    v10 = (sub_16C9C(v5, 0) + 4) / 5u - *(unsigned __int16 *)(a2 + 38);
  }
  sub_16D78(v5, v10);
  return *(_DWORD *)(a2 + 28) == *(_DWORD *)(a1 + 8);
}

