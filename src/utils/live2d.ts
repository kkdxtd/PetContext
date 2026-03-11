import { Cubism4ModelSettings, Live2DModel } from 'pixi-live2d-display'
import { Application, Ticker } from 'pixi.js'

Live2DModel.registerTicker(Ticker)

const MODEL_PATH = '/models/standard'

class Live2d {
  private app: Application | null = null
  public model: Live2DModel | null = null

  constructor() {}

  private initApp() {
    if (this.app) return

    const view = document.getElementById('live2dCanvas') as HTMLCanvasElement

    this.app = new Application({
      view,
      resizeTo: window,
      backgroundAlpha: 0,
      resolution: devicePixelRatio,
    })
  }

  public async load(_path: string) {
    this.initApp()

    this.destroy()

    console.log('Loading model from public folder...')
    
    const modelUrl = MODEL_PATH + '/cat.model3.json'
    console.log('Model URL:', modelUrl)
    
    const response = await fetch(modelUrl)
    const modelJSON = await response.json()
    console.log('Model JSON loaded')

    const modelSettings = new Cubism4ModelSettings({
      ...modelJSON,
      url: modelUrl,
    })

    modelSettings.replaceFiles((file: string) => {
      return MODEL_PATH + '/' + file
    })

    console.log('Creating Live2D model...')
    this.model = await Live2DModel.from(modelSettings)
    console.log('Model created successfully')

    this.app?.stage.addChild(this.model)

    const { width, height } = this.model

    return {
      width,
      height,
    }
  }

  public destroy() {
    this.model?.destroy()
  }

  public resizeModel(modelSize: { width: number; height: number }) {
    if (!this.model) return

    const { width, height } = modelSize

    const scaleX = innerWidth / width
    const scaleY = innerHeight / height
    const scale = Math.min(scaleX, scaleY)

    this.model.scale.set(scale)
    this.model.x = innerWidth / 2
    this.model.y = innerHeight / 2
    this.model.anchor.set(0.5)
  }

  public getCoreModel() {
    const internalModel = this.model?.internalModel as any
    return internalModel?.coreModel
  }

  public getParameterRange(id: string) {
    const coreModel = this.getCoreModel()
    if (!coreModel) return { min: 0, max: 1 }
    
    try {
      const index = coreModel.getParameterIndex(id)
      if (index === -1) return { min: 0, max: 1 }
      const min = coreModel.getParameterMinimumValue(index)
      const max = coreModel.getParameterMaximumValue(index)
      return { min, max }
    } catch {
      return { min: 0, max: 1 }
    }
  }

  public setParameterValue(id: string, value: number | boolean) {
    const coreModel = this.getCoreModel()
    if (!coreModel) return
    
    try {
      return coreModel.setParameterValueById?.(id, Number(value))
    } catch (e) {
      console.warn('Failed to set parameter:', id, e)
    }
  }
}

const live2d = new Live2d()

export default live2d
