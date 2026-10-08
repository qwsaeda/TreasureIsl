/* ea:   0x00045458 */
/* name: WinMain */
/* size: 244 */
int __stdcall WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPWSTR lpCmdLine, int nShowCmd)
{
  int v4; // r0
  int v5; // r0
  int v6; // r4
  int (**v8)(); // [sp+0h] [bp-ACh] BYREF
  void *v9; // [sp+4h] [bp-A8h] BYREF
  void *v10[16]; // [sp+10h] [bp-9Ch] BYREF
  _BYTE v11[92]; // [sp+50h] [bp-5Ch] BYREF

  dword_81218 = (int)&unk_7D218;
  dword_811A8 = 132;
  dword_81030 = (int)&dword_811A8;
  dword_81200 = 768;
  sub_456F8(&dword_811A8, 0, hInstance, nShowCmd);
  sub_45914(&v8);
  sub_3D430((int)v11);
  v10[7] = &off_6FD88;
  v8 = off_6FD60;
  v4 = sub_12F04(aTreasureIsland);
  sub_1318C(&v9, (char *)aTreasureIsland, (int)&aTreasureIsland[v4]);
  v5 = sub_12F04(aTreasureIsland_0);
  sub_1318C(v10, (char *)aTreasureIsland_0, (int)&aTreasureIsland_0[v5]);
  v6 = -1;
  if ( (unsigned __int8)sub_62A14(&v8) != 0 )
    v6 = sub_62B84(&v8);
  sub_457B0(&dword_811A8);
  sub_3D49C((int)v11);
  sub_45C68(&v8);
  return v6;
}

