/* ea:   0x0001B278 */
/* name: sub_1B278 */
/* size: 52 */
int __fastcall sub_1B278(int result, int a2)
{
  *(_DWORD *)(result + 20) = a2;
  *(_DWORD *)result = off_6C97C;
  *(_WORD *)(result + 14) = 0;
  *(_WORD *)(result + 16) = 0;
  *(_DWORD *)(result + 4) = 0;
  *(_DWORD *)(result + 8) = 0;
  *(_WORD *)(result + 12) = 0;
  return result;
}

