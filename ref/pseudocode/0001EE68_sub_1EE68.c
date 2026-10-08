/* ea:   0x0001EE68 */
/* name: sub_1EE68 */
/* size: 28 */
int __fastcall sub_1EE68(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

