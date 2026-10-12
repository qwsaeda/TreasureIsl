/* ea:   0x0005162C */
/* name: sub_5162C */
/* size: 448 */
int __fastcall sub_5162C(int *a1, int *a2)
{
  int result; // r0
  int v5; // r8
  int v6; // r5
  int v7; // r7
  int v8; // r9
  int v9; // r2
  unsigned int v10; // r1
  char v11; // r2
  int v12; // r0
  int v13; // r4
  int v14; // [sp+0h] [bp-28h] BYREF

  result = a1[5];
  if ( result != 0 )
  {
    v5 = 0;
    v6 = 1;
    v7 = 0;
    v8 = 2;
    switch ( *(_BYTE *)(result + 49) )
    {
      case 1:
        v6 = 1;
        break;
      case 2:
        v6 = 2;
        break;
      case 3:
        v6 = 4;
        break;
      case 4:
        v6 = 8;
        break;
      default:
        break;
    }
    if ( *(_BYTE *)(result + 50) == 1 )
    {
      v9 = a1[6] + 288;
    }
    else
    {
      v14 = *(unsigned __int8 *)(result + 50);
      v9 = *(_DWORD *)sub_1DD10((_DWORD *)(a1[6] + 496), &v14);
    }
    v10 = *(unsigned __int8 *)(a1[5] + 48);
    if ( v10 >= 0xA )
      v11 = 0;
    else
      v11 = *(_BYTE *)(*(_DWORD *)(v9 + 188) + v10);
    switch ( v11 )
    {
      case 1:
        v7 = 0;
        break;
      case 2:
        v7 = 30;
        break;
      case 3:
        v7 = 60;
        break;
      case 4:
        v7 = 90;
        break;
      case 5:
        v7 = 140;
        break;
      default:
        break;
    }
    v12 = a1[44];
    switch ( v12 )
    {
      case 2:
        v5 = 10;
        break;
      case 4:
        v5 = 20;
        break;
      case 7:
        v5 = -50;
        break;
      default:
        break;
    }
    v13 = 30 * sub_31150(a1[5], 8);
    if ( !sub_453BC(a1[6] + 144, *(_BYTE *)(a1[5] + 48)) )
      v8 = 1;
    result = *(unsigned __int16 *)(a1[5] + 26) * *(_DWORD *)(a1[6] + 188) * v6 / 100 * (v13 + v5 + v7 + 100) / 100 * v8
           + 1;
    *a2 = result;
  }
  return result;
}

