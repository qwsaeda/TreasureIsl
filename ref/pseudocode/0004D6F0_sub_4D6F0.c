/* ea:   0x0004D6F0 */
/* name: sub_4D6F0 */
/* size: 344 */
BOOL __fastcall sub_4D6F0(int a1, unsigned __int16 a2)
{
  unsigned __int16 *v4; // r9
  unsigned int v5; // r0
  int v6; // r8
  int v7; // r5
  unsigned int v8; // r1
  int v9; // r5
  unsigned __int8 v10; // r0
  unsigned int v11; // r7
  unsigned int v12; // r2
  unsigned __int16 v13; // r1

  v4 = (unsigned __int16 *)(a1 + 136);
  v5 = (unsigned __int16)sub_16C9C((unsigned __int16 *)(a1 + 136), 0) << 16;
  if ( a2 == 0 )
    a2 = 1;
  v6 = 10 * a2;
  v7 = 10 * HIWORD(v5);
  v8 = sub_16C9C(v4, 1) << 16;
  v9 = v7 - 2 * a2;
  v10 = 2;
  if ( *(_DWORD *)(a1 + 176) != 8 )
    v10 = 1;
  v11 = HIWORD(v8);
  if ( v9 <= 0 )
  {
LABEL_10:
    v13 = sub_16C9C(v4, 0);
  }
  else
  {
    while ( 1 )
    {
      v12 = (unsigned __int16)(v9 / 10 + 1);
      if ( v12 <= (unsigned __int16)v11 )
        LOWORD(v11) = v9 / 10 + 1;
      v6 -= 2 * (v12 + (unsigned __int16)v11) * v10;
      if ( v6 <= 0 )
        break;
      v9 -= 2 * (unsigned __int16)(v6 / 10 + 1);
      if ( v9 <= 0 )
        goto LABEL_10;
    }
    v13 = sub_16C9C(v4, 0) - (v9 / 10 + 1);
  }
  sub_16D78(v4, v13);
  return v9 > v6;
}

