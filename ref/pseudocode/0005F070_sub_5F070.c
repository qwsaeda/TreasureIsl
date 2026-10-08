/* ea:   0x0005F070 */
/* name: sub_5F070 */
/* size: 200 */
int __fastcall sub_5F070(int a1, int *a2)
{
  int v4; // r1
  DWORD v6; // r4
  int v7; // r0
  int v8; // r4
  HANDLE v9[5]; // [sp+0h] [bp-2Ch] BYREF
  int v10; // [sp+14h] [bp-18h]

  sub_196AC((_DWORD *)(a1 + 4));
  v4 = *a2;
  v9[1] = (HANDLE)-1;
  v10 = 0;
  v9[0] = off_6D220;
  v9[2] = nullptr;
  if ( (unsigned __int8)sub_61F98(v9, v4, 0, 0) != 0 )
  {
    v6 = sub_1E67C(v9);
    if ( (*((unsigned __int8 (__fastcall **)(HANDLE *))v9[0] + 9))(v9) == 0 )
      __debugbreak();
    v7 = sub_5F138(a1, (char *)v9[3] + v10, v6);
    v9[0] = off_6D220;
    v8 = v7;
    sub_620B0(v9);
    return v8;
  }
  else
  {
    v9[0] = off_6D220;
    sub_620B0(v9);
    return 0;
  }
}

