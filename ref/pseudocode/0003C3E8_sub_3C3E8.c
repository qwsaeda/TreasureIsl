/* ea:   0x0003C3E8 */
/* name: sub_3C3E8 */
/* size: 196 */
int __fastcall sub_3C3E8(int a1, int a2, unsigned __int16 a3, int a4)
{
  int v6; // r6
  int v7; // r0
  unsigned int v8; // r1
  int v9; // r0
  unsigned int v10; // r4
  int v11; // r0
  int v13; // [sp+4h] [bp-18h] BYREF

  v6 = a3;
  v13 = a3;
  v7 = sub_2F944((_DWORD *)(a4 + 520), &v13);
  v8 = *(unsigned __int16 *)(*(_DWORD *)v7 + 16) + 1;
  if ( v8 > 9 )
    LOWORD(v8) = 9;
  *(_WORD *)(*(_DWORD *)v7 + 16) = v8;
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  v9 = sub_12F04(aT);
  v10 = sub_3B0C0(dword_81184, (int)aT, 0, v9);
  v11 = sub_186F0(a4, v6);
  return sub_3AF0C(dword_81184, v10, 2, v11);
}

