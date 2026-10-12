/* ea:   0x0004F090 */
/* name: sub_4F090 */
/* size: 548 */
int __fastcall sub_4F090(int result, int a2, int a3)
{
  _DWORD *v3; // r7
  int v4; // r2
  int v5; // r0
  int v6; // r0
  int v7; // r0
  int v8; // r1
  int v9; // r5
  _DWORD *v10; // r4
  _DWORD *v11; // r0
  _DWORD *v12; // r0

  v3 = (_DWORD *)result;
  if ( a2 == 0 )
  {
    switch ( a3 )
    {
      case 13:
        result = *(_DWORD *)(result + 1096);
        if ( result == 7 )
        {
          v3[252] = 1;
          v3[255] = 1;
        }
        else if ( result == 8 )
        {
          v3[252] = 1;
          v3[255] = 0;
          return 1;
        }
        else
        {
          v9 = 0;
          v10 = v3;
          do
          {
            if ( v3[275] == v10[256] )
            {
              v11 = sub_445F4((int)v3, v9 + 1);
              v3[276] = v11;
              v11[5] = 0;
            }
            ++v9;
            ++v10;
          }
          while ( v9 < 6 );
          v12 = sub_445F4((int)v3, v3[274]);
          v3[276] = v12;
          v12[5] = 1;
          result = v3[274];
          v3[275] = v3[result + 255];
        }
        break;
      case 37:
        v6 = *(_DWORD *)(result + 1096);
        switch ( v6 )
        {
          case 7:
            v8 = v3[276];
            result = v3[277];
            v3[274] = result;
            if ( v8 != 0 )
            {
              *(_DWORD *)(v8 + 20) = 1;
              v3[276] = 0;
              return 1;
            }
            break;
          case 8:
            v7 = v3[276];
            if ( v7 != 0 )
            {
              *(_DWORD *)(v7 + 20) = 1;
              v3[276] = 0;
            }
            result = (int)sub_445F4((int)v3, 12);
            v3[276] = result;
            *(_DWORD *)(result + 20) = 0;
            v3[274] = 7;
            break;
          case 1:
            v3[274] = 8;
            result = (int)sub_445F4((int)v3, 22);
            v3[276] = result;
            *(_DWORD *)(result + 20) = 0;
            break;
          default:
            result = v6 - 1;
            v3[274] = result;
            break;
        }
        break;
      case 39:
        v4 = *(_DWORD *)(result + 1096);
        if ( v4 == 7 )
        {
          *(_DWORD *)(result + 1096) = 8;
          v5 = *(_DWORD *)(result + 1104);
          if ( v5 != 0 )
          {
            *(_DWORD *)(v5 + 20) = 1;
            v3[276] = 0;
          }
          result = (int)sub_445F4((int)v3, 22);
          v3[276] = result;
          *(_DWORD *)(result + 20) = 0;
        }
        else if ( v4 == 8 )
        {
          result = *(_DWORD *)(result + 1104);
          if ( result != 0 )
          {
            *(_DWORD *)(result + 20) = 1;
            v3[276] = 0;
          }
          v3[274] = 1;
        }
        else if ( *(_DWORD *)(result + 4 * v4 + 1072) != 0 )
        {
          *(_DWORD *)(result + 1096) = v4 + 1;
          return v4 + 1;
        }
        else
        {
          *(_DWORD *)(result + 1096) = 7;
          result = (int)sub_445F4(result, 12);
          v3[276] = result;
          *(_DWORD *)(result + 20) = 0;
        }
        break;
      default:
        break;
    }
  }
  return result;
}

