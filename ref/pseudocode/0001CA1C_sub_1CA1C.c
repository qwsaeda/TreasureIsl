/* ea:   0x0001CA1C */
/* name: sub_1CA1C */
/* size: 476 */
int __fastcall sub_1CA1C(_DWORD *a1, int *a2)
{
  int v4; // r0
  int v5; // r1
  int v6; // r2
  int v7; // r3
  int v8; // r1
  int v9; // r2
  int v10; // r3
  int result; // r0

  v4 = *a2;
  v5 = a2[1];
  v6 = a2[2];
  v7 = a2[3];
  a1[255] = v4;
  a1[256] = v5;
  a1[257] = v6;
  a1[258] = v7;
  v8 = a2[5];
  v9 = a2[6];
  v10 = a2[7];
  a1[259] = a2[4];
  a1[260] = v8;
  a1[261] = v9;
  a1[262] = v10;
  result = a1[255];
  if ( result != 0 )
  {
    if ( result == 1 )
    {
      *(_DWORD *)(sub_445F4(a1, 111) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 333) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 666) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 777) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 501) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 502) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 601) + 20) = 0;
      result = sub_445F4(a1, 602);
      *(_DWORD *)(result + 20) = 0;
    }
    else if ( result == 2 )
    {
      *(_DWORD *)(sub_445F4(a1, 111) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 222) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 666) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 777) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 501) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 502) + 20) = 0;
      *(_DWORD *)(sub_445F4(a1, 601) + 20) = 0;
      result = sub_445F4(a1, 602);
      *(_DWORD *)(result + 20) = 0;
    }
  }
  else
  {
    *(_DWORD *)(sub_445F4(a1, 222) + 20) = 0;
    result = sub_445F4(a1, 333);
    *(_DWORD *)(result + 20) = 0;
  }
  return result;
}

